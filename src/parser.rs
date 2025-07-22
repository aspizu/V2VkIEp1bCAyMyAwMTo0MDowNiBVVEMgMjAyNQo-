use crate::{ast, tokens::Token};

#[derive(Copy, Clone)]
enum SubShellKind {
    CmdSubst,
    Normal,
}

impl Into<Token> for SubShellKind {
    fn into(self) -> Token {
        match self {
            Self::CmdSubst => Token::CmdSubstEnd,
            Self::Normal => Token::CloseParen,
        }
    }
}

#[derive(Copy,Clone)]
enum IfClauseTok {
    If,
    Else,
    Elif,
    Then,
    Fi,
}

impl From<&'_ Token> for Option<IfClauseTok> {
    fn from(token: &'_ Token) -> Self {
        if let Token::Text(text) = token {
            match text.as_str() {
                "if" => Some(IfClauseTok::If),
                _ => None
            }
        } else { 
            None
        }
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    inside_subshell: Option<SubShellKind>,
}

impl<'a> Parser<'a> {
    fn make_subparser(&mut self, kind: SubShellKind) -> Self {
        Self {
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

    fn parse(&mut self) -> ast::Script {
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
            return self.parse_if_clause();
        }

        if self.peek() == &Token::DoubleBracketOpen {
            return self.parse_cond_expr();
        }

        self.parse_simple_cmd()
    }

    fn parse_subshell(&mut self) -> ast::SubShell {
        self.expect(&Token::OpenParen);
        let subparser = self.make_subparser(SubShellKind::Normal);
        let script = subparser.parse();
        self.continue_from_subparser(subparser);
        let parsed_redirect = self.parse_redirect();
        ast::SubShell {
            script,
            redirect: parser_redirect.redirect,
            redirect_flags: parsed_redirect.flags,
        }
    }

    fn parse_if_body(&mut self, )

    fn parse_if_clause(&mut self) -> ast::If {
        self.except_if_clause_text_token("if");
        let cond = self.parse_if_body();
    }

    fn except_if_clause_text_token(&mut self, if_clause_token: &str) -> &Token {
        if let Token::Text(text) = self.peek() {
            if self.delimits(self.peek_n(1)) && text == if_clause_token {
                let tok = self.advance();
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
        return false;
    }

    fn check_any(&self, tokens: &[&Token]) -> bool {
        tokens.into_iter().map(|token| self.check(token)).any(|b| b)
    }

    fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn peek_n(&mut self, n: usize) -> &Token {
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
