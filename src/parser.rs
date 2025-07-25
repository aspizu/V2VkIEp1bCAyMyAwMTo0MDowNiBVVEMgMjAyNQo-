use bytes::BytesMut;

use crate::{ast, tokens::Token};

#[derive(Copy, Clone)]
enum SubShellKind {
    CmdSubst,
    Normal,
}

impl From<SubShellKind> for Token {
    fn from(val: SubShellKind) -> Self {
        match val {
            SubShellKind::CmdSubst => Token::CmdSubstEnd,
            SubShellKind::Normal => Token::CloseParen,
        }
    }
}

#[derive(Copy, Clone)]
enum IfClauseTok {
    If,
    Else,
    Elif,
    Then,
    Fi,
}

impl IfClauseTok {
    fn parse(token: &Token, arena: &[u8]) -> Option<Self> {
        if let Token::Text(text) = token {
            match &**text {
                b"if" => Some(Self::If),
                b"else" => Some(Self::Else),
                b"elif" => Some(Self::Elif),
                b"then" => Some(Self::Then),
                b"fi" => Some(Self::Fi),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl From<&IfClauseTok> for &str {
    fn from(tok: &IfClauseTok) -> Self {
        match tok {
            IfClauseTok::If => "if",
            IfClauseTok::Else => "else",
            IfClauseTok::Elif => "elif",
            IfClauseTok::Then => "then",
            IfClauseTok::Fi => "fi",
        }
    }
}

struct ParsedRedirect {
    redirect: Option<ast::Redirect>,
    flags: ast::RedirectFlags,
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    inside_subshell: Option<SubShellKind>,
    arena: &'a [u8],
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], arena: &'a [u8]) -> Self {
        Self {
            tokens,
            arena,
            current: 0,
            inside_subshell: None,
        }
    }

    fn make_subparser(&mut self, kind: SubShellKind) -> Self {
        Self {
            arena: self.arena,
            tokens: self.tokens,
            current: self.current,
            inside_subshell: Some(kind),
        }
    }

    fn continue_from_subparser(&mut self, subparser: Parser) {
        self.current = if subparser.current >= self.tokens.len() {
            subparser.current
        } else {
            subparser.current + 1
        };
    }

    pub fn parse(&mut self) -> ast::Script {
        let mut stmts: Vec<ast::Stmt> = vec![];
        if self.tokens.is_empty() || self.tokens.len() == 1 && matches!(self.tokens[0], Token::Eof)
        {
            return ast::Script { stmts: vec![] };
        }
        while if self.inside_subshell.is_none() {
            !self.matches(&Token::Eof)
        } else {
            !self.matches_any(&[&Token::Eof, &self.inside_subshell.unwrap().into()])
        } {
            self.skip_newlines();
            stmts.push(self.parse_stmt());
            self.skip_newlines();
        }
        if let Some(kind) = self.inside_subshell {
            self.expect_any(&[&Token::Eof, &kind.into()]);
        } else {
            self.expect(&Token::Eof);
        }
        ast::Script { stmts }
    }

    fn parse_stmt(&mut self) -> ast::Stmt {
        let mut exprs: Vec<ast::Expr> = vec![];

        while if self.inside_subshell.is_none() {
            !self.matches_any(&[&Token::Semicolon, &Token::Newline, &Token::Eof])
        } else {
            !self.matches_any(&[
                &Token::Semicolon,
                &Token::Newline,
                &Token::Eof,
                &self.inside_subshell.unwrap().into(),
            ])
        } {
            let expr = self.parse_expr();
            if self.matches(&Token::Ampersand) {
                panic!("Background commands \"&\" are not supported yet.");
            }
            exprs.push(expr);
        }

        ast::Stmt { exprs }
    }

    fn parse_expr(&mut self) -> ast::Expr {
        let mut left = self.parse_pipeline();
        while self.matches_any(&[&Token::DoubleAmpersand, &Token::DoublePipe]) {
            let op = match self.prev() {
                Token::DoubleAmpersand => ast::Op::And,
                Token::DoublePipe => ast::Op::Or,
                _ => unreachable!(),
            };
            let right = self.parse_pipeline();
            let binary = ast::Binary { op, left, right };
            left = ast::Expr::Binary(Box::new(binary));
        }
        left
    }

    fn parse_pipeline(&mut self) -> ast::Expr {
        let mut expr = self.parse_compound_cmd();

        if self.peek() == &Token::Pipe {
            let mut pipeline_items: Vec<ast::PipelineItem> = vec![];
            if let Some(pipeline_item) = expr.as_pipeline_item() {
                pipeline_items.push(pipeline_item);
            } else {
                // TODO: Add error handling
                panic!("Expected pipeline item")
            }
            while self.matches(&Token::Pipe) {
                expr = self.parse_compound_cmd();
                if let Some(pipeline_item) = expr.as_pipeline_item() {
                    pipeline_items.push(pipeline_item);
                } else {
                    // TODO: Add error handling
                    panic!("Expected pipeline item")
                }
            }
            return ast::Expr::Pipeline(ast::Pipeline {
                items: pipeline_items,
            });
        }

        expr
    }

    fn parse_compound_cmd(&mut self) -> ast::Expr {
        // Placeholder for when we fully support subshells
        if self.peek() == &Token::OpenParen {
            let subshell = self.parse_subshell();
            if !subshell.redirect_flags.is_empty() {
                panic!("Subshells with redirections are currently not supported. Please open a GitHub issue.");
            }
            return ast::Expr::SubShell(subshell);
        }

        if self.is_if_clause_text_token("if") {
            return ast::Expr::If(self.parse_if_clause());
        }

        if self.peek() == &Token::DoubleBracketOpen {
            return ast::Expr::CondExpr(self.parse_cond_expr());
        }

        match self.parse_simple_cmd() {
            ast::CmdOrAssigns::Cmd(cmd) => ast::Expr::Cmd(cmd),
            ast::CmdOrAssigns::Assigns(assigns) => ast::Expr::Assign(assigns),
        }
    }

    fn parse_subshell(&mut self) -> ast::SubShell {
        self.expect(&Token::OpenParen);
        let mut subparser = self.make_subparser(SubShellKind::Normal);
        let script = subparser.parse();
        self.continue_from_subparser(subparser);
        let parsed_redirect = self.parse_redirect();
        ast::SubShell {
            script,
            redirect: parsed_redirect.redirect,
            redirect_flags: parsed_redirect.flags,
        }
    }

    fn parse_if_body(&mut self, until: &[IfClauseTok]) -> Vec<ast::Stmt> {
        let mut ret = vec![];
        while if self.inside_subshell.is_none() {
            !self.peek_any_ifclausetok(until) && !self.peek_any(&[&Token::Eof])
        } else {
            !self.peek_any_ifclausetok(until)
                && !self.peek_any(&[&self.inside_subshell.unwrap().into(), &Token::Eof])
        } {
            self.skip_newlines();
            let stmt = self.parse_stmt();
            ret.push(stmt);
            self.skip_newlines();
        }
        ret
    }

    fn parse_if_clause(&mut self) -> ast::If {
        self.expect_if_clause_text_token("if");
        let cond = self.parse_if_body(&[IfClauseTok::Then]);
        if !self.match_if_clausetok(IfClauseTok::Then) {
            // TODO: Add error handling
            panic!("Expected \"then\" but got: {:?}", self.peek());
        }
        let then = self.parse_if_body(&[IfClauseTok::Else, IfClauseTok::Elif, IfClauseTok::Fi]);
        let mut else_parts: Vec<Vec<ast::Stmt>> = vec![];

        let if_clause_tok: IfClauseTok = match IfClauseTok::parse(self.peek(), self.arena) {
            Some(tok) => tok,
            None => {
                // TODO: add error handling
                panic!(
                    "Expected \"else\", \"elif\", or \"fi\" but got: {:?}",
                    self.peek()
                );
            }
        };

        match if_clause_tok {
            IfClauseTok::If | IfClauseTok::Then => {
                // TODO: add error handling
                panic!(
                    "Expected \"else\", \"elif\", or \"fi\" but got: {:?}",
                    self.peek()
                );
            }
            IfClauseTok::Else => {
                self.expect_if_clause_text_token("else");
                let else_part = self.parse_if_body(&[IfClauseTok::Fi]);
                if !self.match_if_clausetok(IfClauseTok::Fi) {
                    panic!("Expected \"fi\" but got: {:?}", self.peek());
                }
                else_parts.push(else_part);
                ast::If {
                    cond,
                    then,
                    else_parts,
                }
            }
            IfClauseTok::Elif => {
                loop {
                    self.expect_if_clause_text_token("elif");
                    let elif_cond = self.parse_if_body(&[IfClauseTok::Then]);
                    if !self.match_if_clausetok(IfClauseTok::Then) {
                        // TODO: add error handling
                        panic!("Expected \"then\" but got: {:?}", self.peek());
                    }
                    let then_part = self.parse_if_body(&[
                        IfClauseTok::Elif,
                        IfClauseTok::Else,
                        IfClauseTok::Fi,
                    ]);
                    else_parts.push(elif_cond);
                    else_parts.push(then_part);

                    match IfClauseTok::parse(self.peek(), self.arena) {
                        Some(IfClauseTok::Elif) => continue,
                        Some(IfClauseTok::Else) => {
                            self.expect_if_clause_text_token("else");
                            let else_part = self.parse_if_body(&[IfClauseTok::Fi]);
                            else_parts.push(else_part);
                            break;
                        }
                        _ => break,
                    }
                }
                if !self.match_if_clausetok(IfClauseTok::Fi) {
                    panic!("Expected \"fi\" but got: {:?}", self.peek());
                }
                ast::If {
                    cond,
                    then,
                    else_parts,
                }
            }
            IfClauseTok::Fi => {
                self.expect_if_clause_text_token("fi");
                ast::If {
                    cond,
                    then,
                    else_parts: vec![],
                }
            }
        }
    }

    fn parse_simple_cmd(&mut self) -> ast::CmdOrAssigns {
        let mut assigns: Vec<ast::Assign> = vec![];

        while if self.inside_subshell.is_none() {
            !self.check_any(&[&Token::Semicolon, &Token::Newline, &Token::Eof])
        } else {
            !self.check_any(&[
                &Token::Semicolon,
                &Token::Newline,
                &Token::Eof,
                &self.inside_subshell.unwrap().into(),
            ])
        } {
            if let Some(assign) = self.parse_assign() {
                assigns.push(assign);
            } else {
                break;
            }
        }

        if if self.inside_subshell.is_none() {
            self.check_any(&[&Token::Semicolon, &Token::Newline, &Token::Eof])
        } else {
            self.check_any(&[
                &Token::Semicolon,
                &Token::Newline,
                &Token::Eof,
                &self.inside_subshell.unwrap().into(),
            ])
        } {
            if assigns.is_empty() {
                // TODO: add error handling
                panic!("expected a command or assignment");
            }
            return ast::CmdOrAssigns::Assigns(assigns);
        }

        let Some(name) = self.parse_atom() else {
            if assigns.is_empty() {
                // TODO: add error handling
                panic!("expected a command or assignment but got smth else");
            }
            return ast::CmdOrAssigns::Assigns(assigns);
        };

        let mut name_and_args: Vec<ast::Atom> = vec![];
        name_and_args.push(name);
        while let Some(arg) = self.parse_atom() {
            name_and_args.push(arg);
        }
        let parsed_redirect = self.parse_redirect();

        ast::CmdOrAssigns::Cmd(ast::Cmd {
            assigns,
            name_and_args,
            redirect: parsed_redirect.redirect,
            redirect_flags: parsed_redirect.flags,
        })
    }

    fn parse_assign(&mut self) -> Option<ast::Assign> {
        let old = self.current;
        if let Token::Text(txt) = self.peek().clone() {
            let start_idx = self.current;
            self.expect_text();
            let var_decl: Option<ast::Assign> = 'var_decl: {
                if let Some((label, value)) = txt.split_once(|c| *c == b'=') {
                    let label = BytesMut::from(label);
                    let value = BytesMut::from(value);
                    // If it starts with = then it's not valid assignment (e.g. `=FOO`)
                    if label.is_empty() {
                        break 'var_decl None;
                    }
                    if !is_valid_var_name(&label) {
                        break 'var_decl None;
                    }
                    if value.is_empty() {
                        if self.delimits(self.peek()) {
                            self.expect_delimit();
                            break 'var_decl Some(ast::Assign {
                                label: label.into(),
                                value: ast::Atom::Simple(ast::SimpleAtom::Text("".into())),
                            });
                        }
                        // TODO: handle error reporting
                        let atom = self.parse_atom().expect("Expected an atom");
                        break 'var_decl Some(ast::Assign {
                            label: label.into(),
                            value: atom,
                        });
                    }
                    if self.delimits(self.peek()) {
                        self.expect_delimit();
                        break 'var_decl Some(ast::Assign {
                            label: label.into(),
                            value: ast::Atom::Simple(ast::SimpleAtom::Text(value.into())),
                        });
                    }
                    // TODO: handle error reporting
                    let right = self.parse_atom().expect("Expected an atom");
                    let left = ast::Atom::Simple(ast::SimpleAtom::Text(value.into()));
                    let merged = ast::Atom::merge(left, right);
                    break 'var_decl Some(ast::Assign {
                        label: label.into(),
                        value: ast::Atom::CompoundAtom(merged),
                    });
                }
                break 'var_decl None;
            };
            if let Some(var_decl) = var_decl {
                return Some(var_decl);
            }
            self.current = start_idx;
            None
        } else {
            None
        }
    }

    fn expect_text(&mut self) {
        if !matches!(self.peek(), Token::Text(_)) {
            panic!("Expected text token")
        }
        self.advance();
    }

    fn expect_var(&mut self) {
        if !matches!(self.peek(), Token::Var(_)) {
            panic!("Expected var token")
        }
        self.advance();
    }

    fn expect_varargv(&mut self) {
        if !matches!(self.peek(), Token::VarArgv(_)) {
            panic!("Expected varargv token")
        }
        self.advance();
    }

    fn parse_atom(&mut self) -> Option<ast::Atom> {
        let mut has_brace_open = false;
        let mut has_brace_close = false;
        let mut has_comma = false;
        let mut has_glob_syntax = false;
        let mut atoms = vec![];

        {
            while match self.peek() {
                Token::Delimit => {
                    self.expect(&Token::Delimit);
                    false
                }
                Token::Eof | Token::Semicolon | Token::Newline => false,
                t => !self
                    .inside_subshell
                    .is_some_and(|kind| Into::<Token>::into(kind) == *t),
            } {
                let next = self.peek_n(1);
                let next_delimits = self.delimits(next);
                let peeked = self.peek().clone();
                let should_break = next_delimits;
                let peeked_is_text = matches!(peeked, Token::Text(_));
                match peeked {
                    Token::Asterisk => {
                        has_glob_syntax = true;
                        self.expect(&Token::Asterisk);
                        atoms.push(ast::SimpleAtom::Asterisk);
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            break;
                        }
                    }
                    Token::DoubleAsterisk => {
                        has_glob_syntax = true;
                        self.expect(&Token::DoubleAsterisk);
                        atoms.push(ast::SimpleAtom::DoubleAsterisk);
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            break;
                        }
                    }
                    Token::BraceBegin => {
                        has_brace_open = true;
                        self.expect(&Token::BraceBegin);
                        atoms.push(ast::SimpleAtom::BraceBegin);
                        // TODO in this case we know it can't possibly be the beginning
                        // of a brace expansion so maybe its faster to just change it to
                        // text here.
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            if should_break {
                                break;
                            }
                        }
                    }
                    Token::BraceEnd => {
                        has_brace_close = true;
                        self.expect(&Token::BraceEnd);
                        atoms.push(ast::SimpleAtom::BraceEnd);
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            break;
                        }
                    }
                    Token::Comma => {
                        has_comma = true;
                        self.expect(&Token::Comma);
                        atoms.push(ast::SimpleAtom::Comma);
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            if should_break {
                                break;
                            }
                        }
                    }
                    Token::CmdSubstBegin => {
                        self.expect(&Token::CmdSubstBegin);
                        let is_quoted = self.matches(&Token::CmdSubstQuoted);
                        let mut subparser = self.make_subparser(SubShellKind::CmdSubst);
                        let script = subparser.parse();
                        atoms.push(ast::SimpleAtom::CmdSubst {
                            script,
                            quoted: is_quoted,
                        });
                        self.continue_from_subparser(subparser);
                        if self.delimits(self.peek()) {
                            self.matches(&Token::Delimit);
                            break;
                        }
                    }
                    Token::SingleQuotedText(text)
                    | Token::DoubleQuotedText(text)
                    | Token::Text(text) => {
                        self.advance();
                        if peeked_is_text && !text.is_empty() && text.starts_with(b"~") {
                            let text = text.slice(1..);
                            atoms.push(ast::SimpleAtom::Tilde);
                            if !text.is_empty() {
                                atoms.push(ast::SimpleAtom::Text(text.into()));
                            }
                        } else {
                            atoms.push(ast::SimpleAtom::Text(text.into()));
                        }
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            if should_break {
                                break;
                            }
                        }
                    }
                    Token::Var(text) => {
                        self.expect_var();
                        atoms.push(ast::SimpleAtom::Var(text.into()));
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            if should_break {
                                break;
                            }
                        }
                    }
                    Token::VarArgv(int) => {
                        self.expect_varargv();
                        atoms.push(ast::SimpleAtom::VarArgv(int));
                        if next_delimits {
                            self.matches(&Token::Delimit);
                            if should_break {
                                break;
                            }
                        }
                    }
                    Token::OpenParen | Token::CloseParen => {
                        panic!("Unexpected parenthesis in atom parsing");
                    }
                    _ => return None,
                }
            }
        }

        match atoms.len() {
            0 => None,
            1 => Some(ast::Atom::Simple(atoms.pop().unwrap())),
            _ => {
                let brace_expansion_hint = has_brace_open && has_brace_close && has_comma;
                let glob_hint = has_glob_syntax;
                Some(ast::Atom::CompoundAtom(ast::CompoundAtom {
                    atoms,
                    brace_expansion_hint,
                    glob_hint,
                }))
            }
        }
    }

    fn parse_redirect(&mut self) -> ParsedRedirect {
        let has_redirect: bool = matches!(self.peek(), Token::Redirect(..));
        let flags: ast::RedirectFlags = if has_redirect {
            if let Token::Redirect(r) = self.advance() {
                *r
            } else {
                unreachable!()
            }
        } else {
            ast::RedirectFlags::default()
        };
        let redirect: Option<ast::Redirect> = if has_redirect {
            if matches!(self.peek(), Token::PyObject(_)) {
                todo!("implement python object redirection")
            } else {
                let Some(redirect_file) = self.parse_atom() else {
                    // TODO: add error handling
                    panic!("redirection with no file");
                };
                Some(ast::Redirect::Atom(redirect_file))
            }
        } else {
            None
        };
        // TODO check for multiple redirects and error
        ParsedRedirect { redirect, flags }
    }

    fn parse_cond_expr(&mut self) -> ast::CondExpr {
        self.expect(&Token::DoubleBracketOpen);
        todo!("Conditional Expressions not implemented yet.")
    }

    fn match_if_clausetok(&mut self, token: IfClauseTok) -> bool {
        let Token::Text(text) = self.peek() else {
            return false;
        };
        let stok: &str = (&token).into();
        if self.delimits(self.peek_n(1)) && text == stok {
            self.advance();
            self.expect_delimit();
            return true;
        }
        false
    }

    fn peek_any_ifclausetok(&self, tokens: &[IfClauseTok]) -> bool {
        let peeked = self.peek();
        let Token::Text(text) = peeked else {
            return false;
        };
        for token in tokens {
            let stok: &str = token.into();
            if text == stok {
                return true;
            }
        }
        false
    }

    fn expect_if_clause_text_token(&mut self, if_clause_token: &str) -> Token {
        if let Token::Text(text) = self.peek() {
            let d = self.delimits(self.peek_n(1));
            let mut x = false;
            let mut tok = Token::Eof;
            if d && text == if_clause_token {
                tok = self.advance().clone();
                x = true;
            }
            if x {
                self.expect_delimit();
                return tok;
            }
        };
        panic!("Expected `{if_clause_token}`");
    }

    fn is_if_clause_text_token(&mut self, if_clause_token: &str) -> bool {
        let Token::Text(text) = self.peek() else {
            return false;
        };
        text == if_clause_token
    }

    fn skip_newlines(&mut self) {
        while self.matches(&Token::Newline) {}
    }

    fn matches(&mut self, token: &Token) -> bool {
        if self.peek() == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches_any(&mut self, tokens: &[&Token]) -> bool {
        for token in tokens {
            if self.matches(token) {
                return true;
            }
        }
        false
    }

    fn check_any(&self, tokens: &[&Token]) -> bool {
        tokens.iter().any(|token| self.check(token))
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn peek_any(&self, tokens: &[&Token]) -> bool {
        tokens.iter().any(|token| self.check(token))
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn peek_n(&self, n: usize) -> &Token {
        if self.current + n >= self.tokens.len() {
            return &self.tokens[self.tokens.len() - 1];
        }
        &self.tokens[self.current + n]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.prev()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
            || self
                .inside_subshell
                .as_ref()
                .is_some_and(|&kind| self.peek() == &kind.into())
    }

    fn prev(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn expect(&mut self, token: &Token) -> &Token {
        if self.peek() != token {
            panic!("Unexpected token");
        }
        self.advance()
    }

    fn expect_any(&mut self, tokens: &[&Token]) -> &Token {
        for token in tokens {
            if self.peek() == *token {
                return self.advance();
            }
        }
        panic!("Unexpected token")
    }

    fn delimits(&self, token: &Token) -> bool {
        token == &Token::Delimit
            || token == &Token::Semicolon
            || token == &Token::Eof
            || token == &Token::Newline
            || self
                .inside_subshell
                .is_some_and(|kind| token == &kind.into())
    }

    fn expect_delimit(&mut self) -> &Token {
        if self.delimits(self.peek()) {
            return self.advance();
        }
        panic!("Expected a delimiter token");
    }
}

fn is_valid_var_name(var_name: &[u8]) -> bool {
    if var_name.is_empty() {
        return false;
    }

    let mut chars = var_name.iter().copied();

    // Check first character
    if let Some(first_char) = chars.next() {
        match first_char {
            b'0'..=b'9' => return false,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {}
            _ => return false,
        }
    } else {
        return false;
    }

    // Check remaining characters
    for c in chars {
        match c {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' => {}
            _ => return false,
        }
    }

    true
}
