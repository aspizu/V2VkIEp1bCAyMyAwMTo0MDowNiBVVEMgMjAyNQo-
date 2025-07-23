use crate::{ast::RedirectFlags, tokens::Token};
use std::{ops::Range, str};

const PLACEHOLDER: u8 = 8;

#[derive(Copy, Clone)]
enum State {
    Normal,
    Single,
    Double,
}

#[derive(Copy, Clone)]
enum SubShellKind {
    Normal,
    Backtick,
    Dollar,
}

pub struct Lexer<'a, 'b> {
    chars: &'a [u8],
    j: usize,
    word_start: usize,
    state: State,
    tokens: &'b mut Vec<Token>,
    prev: Option<InputChar>,
    current: Option<InputChar>,
    delimit_quote: bool,
    in_subshell: Option<SubShellKind>,
}

struct BacktrackSnapshot {
    state: State,
    prev: Option<InputChar>,
    current: Option<InputChar>,
    j: usize,
    word_start: usize,
    delimit_quote: bool,
}

#[derive(Copy, Clone)]
struct InputChar {
    char: u8,
    escaped: bool,
}

impl<'a, 'b> Lexer<'a, 'b> {
    pub fn new(chars: &'a [u8], tokens: &'b mut Vec<Token>) -> Self {
        Self {
            chars,
            j: 0,
            word_start: 0,
            state: State::Normal,
            tokens,
            prev: None,
            current: None,
            delimit_quote: false,
            in_subshell: None,
        }
    }

    pub fn lex(&mut self) {
        'l: loop {
            let Some(input) = self.eat() else {
                self.break_word(true);
                break;
            };
            let char = input.char;
            let escaped = input.escaped;
            // Special token to denote substituted JS variables
            // we use 8 or \b which is a non printable char
            if char == PLACEHOLDER {
                todo!()
            }
            // Handle non-escaped chars:
            // 1. special syntax (operators, etc.)
            // 2. lexing state switchers (quotes)
            // 3. word breakers (spaces, etc.)
            else if !escaped {
                'escaped: {
                    match char {
                        b'[' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            let Some(p) = self.peek() else {
                                break 'escaped;
                            };
                            if p.escaped || p.char != b'[' {
                                break 'escaped;
                            }
                            let state = self.make_snapshot();
                            self.eat();
                            'do_backtrack: {
                                let Some(p2) = self.peek() else {
                                    self.break_word(true);
                                    self.tokens.push(Token::DoubleBracketClose);
                                    continue 'l;
                                };
                                if p2.escaped {
                                    break 'do_backtrack;
                                }
                                match p2.char {
                                    b' ' | b'\r' | b'\n' | b'\t' => {
                                        self.break_word(true);
                                        self.tokens.push(Token::DoubleBracketOpen);
                                    }
                                    _ => break 'do_backtrack,
                                }
                                continue 'l;
                            }
                            self.backtrack(state);
                        }
                        b']' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            let Some(p) = self.peek() else {
                                break 'escaped;
                            };
                            if p.escaped || p.char != b']' {
                                break 'escaped;
                            }
                            let state = self.make_snapshot();
                            self.eat();
                            'do_backtrack: {
                                let Some(p2) = self.peek() else {
                                    self.break_word(true);
                                    self.tokens.push(Token::DoubleBracketClose);
                                    continue 'l;
                                };
                                if p2.escaped {
                                    break 'do_backtrack;
                                }
                                match p2.char {
                                    b' ' | b'\r' | b'\n' | b'\t' | b';' | b'&' | b'|' | b'>' => {
                                        self.break_word(true);
                                        self.tokens.push(Token::DoubleBracketClose);
                                    }
                                    _ => break 'do_backtrack,
                                }
                                continue 'l;
                            }
                            self.backtrack(state);
                        }
                        b'#' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            let whitespace_preceding = if let Some(prev) = self.prev {
                                prev.char.is_ascii_whitespace()
                            } else {
                                true
                            };
                            if !whitespace_preceding {
                                break 'escaped;
                            };
                            self.break_word(true);
                            self.eat_comment();
                            continue 'l;
                        }
                        b';' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word(true);
                            self.tokens.push(Token::Semicolon);
                            continue 'l;
                        }
                        b'\n' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word_impl(true, true, false);
                            self.tokens.push(Token::Newline);
                            continue 'l;
                        }
                        b'*' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            if let Some(next) = self.peek() {
                                if !next.escaped && next.char == b'*' {
                                    self.eat();
                                    self.break_word(false);
                                    self.tokens.push(Token::DoubleAsterisk);
                                    continue 'l;
                                }
                            }
                            self.break_word(false);
                            self.tokens.push(Token::Asterisk);
                            continue 'l;
                        }
                        b'{' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word(false);
                            self.tokens.push(Token::BraceBegin);
                            continue 'l;
                        }
                        b',' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word(false);
                            self.tokens.push(Token::Comma);
                            continue 'l;
                        }
                        b'}' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word(false);
                            self.tokens.push(Token::BraceEnd);
                            continue 'l;
                        }
                        b'`' => {
                            if matches!(self.state, State::Single) {
                                break 'escaped;
                            }
                            if matches!(self.in_subshell, Some(SubShellKind::Backtick)) {
                                self.break_word_operator();
                                if let Some(last) = self.tokens.last() {
                                    if !matches!(last, Token::Delimit) {
                                        self.tokens.push(Token::Delimit);
                                    }
                                    return;
                                }
                            } else {
                                self.eat_subshell(SubShellKind::Backtick);
                            }
                        }
                        b'$' => {
                            if matches!(self.state, State::Single) {
                                break 'escaped;
                            }
                            let peeked = self.peek().unwrap_or(InputChar {
                                char: 0,
                                escaped: false,
                            });
                            if !peeked.escaped && peeked.char == b'(' {
                                self.break_word(false);
                                self.eat_subshell(SubShellKind::Dollar);
                                continue 'l;
                            }
                            self.break_word(false);
                            let var_tok = self.eat_var();
                            match var_tok.len() {
                                0 => {
                                    self.break_word(false);
                                }
                                1 => {
                                    let c = self.chars[var_tok.start];
                                    if c.is_ascii_digit() {
                                        self.tokens.push(Token::VarArgv(c - b'0'));
                                    } else {
                                        self.tokens.push(Token::Var(
                                            str::from_utf8(&self.chars[var_tok]).unwrap().into(),
                                        ));
                                    }
                                }
                                _ => {
                                    self.tokens.push(Token::Var(
                                        str::from_utf8(&self.chars[var_tok]).unwrap().into(),
                                    ));
                                }
                            }
                            self.word_start = self.j;
                            continue 'l;
                        }
                        b'(' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            self.break_word(true);
                            self.eat_subshell(SubShellKind::Normal);
                            continue 'l;
                        }
                        b')' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            };
                            if !matches!(
                                self.in_subshell,
                                Some(SubShellKind::Dollar) | Some(SubShellKind::Normal)
                            ) {
                                panic!("Unexpected `)`"); // TODO: handle errors!
                            }
                            self.break_word(true);
                            if matches!(self.in_subshell, Some(SubShellKind::Dollar)) {
                                if let Some(tok) = self.tokens.last() {
                                    match tok {
                                        Token::Delimit
                                        | Token::Semicolon
                                        | Token::Eof
                                        | Token::Newline => {}
                                        _ => {
                                            self.tokens.push(Token::Delimit);
                                        }
                                    }
                                }
                            }

                            if matches!(self.in_subshell, Some(SubShellKind::Dollar)) {
                                self.tokens.push(Token::CmdSubstEnd);
                            } else if matches!(self.in_subshell, Some(SubShellKind::Normal)) {
                                self.tokens.push(Token::CloseParen);
                            }
                            return;
                        }
                        b'0'..=b'9' => {
                            if matches!(self.state, State::Normal) {
                                break 'escaped;
                            }
                            let snapshot = self.make_snapshot();
                            if let Some(redirect) = self.eat_redirect(input) {
                                self.break_word(true);
                                self.tokens.push(Token::Redirect(redirect));
                                continue 'l;
                            }
                            self.backtrack(snapshot);
                            break 'escaped;
                        }
                        b'|' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            }
                            self.break_word_operator();
                            let Some(next) = self.peek() else {
                                panic!("Unexpected EOF") // TODO: Add error handling
                            };
                            if !next.escaped && next.char == b'&' {
                                // TODO: Add error handling
                                panic!("Piping stdout and stderr (`|&`) is not supported yet. Please file an issue on GitHub.")
                            }
                            if next.escaped || next.char != b'|' {
                                self.tokens.push(Token::Pipe);
                            } else if next.char == b'|' {
                                self.eat().unwrap();
                                self.tokens.push(Token::DoublePipe);
                            }
                            continue 'l;
                        }
                        b'>' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            }
                            self.break_word_operator();
                            let redirect = self.eat_simple_redirect(false);
                            self.tokens.push(Token::Redirect(redirect));
                            continue 'l;
                        }
                        b'<' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            }
                            self.break_word_operator();
                            let redirect = self.eat_simple_redirect(true);
                            self.tokens.push(Token::Redirect(redirect));
                            continue 'l;
                        }
                        b'&' => {
                            if matches!(self.state, State::Single | State::Double) {
                                break 'escaped;
                            }
                            self.break_word_operator();
                            let Some(next) = self.peek() else {
                                self.tokens.push(Token::Ampersand);
                                continue 'l;
                            };
                            if next.char == b'>' && !next.escaped {
                                self.eat();
                                let inner = if self.eat_simple_redirect_operator(false) {
                                    RedirectFlags::andrightright()
                                } else {
                                    RedirectFlags::andright()
                                };
                                self.tokens.push(Token::Redirect(inner));
                            } else if next.escaped || next.char != b'&' {
                                self.tokens.push(Token::Ampersand);
                            } else if next.char == b'&' {
                                self.eat().unwrap();
                                self.tokens.push(Token::DoubleAmpersand);
                            } else {
                                self.tokens.push(Token::Ampersand);
                                continue 'l;
                            }
                        }
                        b'\'' => {
                            match self.state {
                                State::Single => {
                                    self.state = State::Normal;
                                }
                                State::Normal => {
                                    self.state = State::Single;
                                }
                                _ => {}
                            }
                            break 'escaped;
                        }
                        b'"' => {
                            match self.state {
                                State::Single => break 'escaped,
                                State::Normal => {
                                    self.break_word(false);
                                    self.state = State::Double;
                                }
                                State::Double => {
                                    self.break_word(false);
                                    self.state = State::Normal;
                                }
                            }
                            continue 'l;
                        }
                        b' ' => {
                            if matches!(self.state, State::Normal) {
                                self.break_word_impl(true, true, false);
                                continue 'l;
                            }
                            break 'escaped;
                        }
                        _ => break 'escaped,
                    }
                    continue 'l;
                }
            }
            // Treat newline preceded by backslash as whitespace
            else if char == b'\n' {
                if !matches!(self.state, State::Double) {
                    self.break_word_impl(true, true, false);
                }
                continue 'l;
            }
            if let Some(subshell_kind) = self.in_subshell {
                match subshell_kind {
                    SubShellKind::Dollar | SubShellKind::Backtick => {
                        panic!("Unclosed command substitution")
                    }
                    SubShellKind::Normal => {
                        panic!("Unclosed subshell")
                    }
                }
            }
        }
        self.tokens.push(Token::Eof);
    }

    fn eat_comment(&mut self) {
        while let Some(peeked) = self.eat() {
            if peeked.escaped {
                continue;
            }
            if peeked.char == b'\n' {
                break;
            }
        }
    }

    fn make_snapshot(&self) -> BacktrackSnapshot {
        BacktrackSnapshot {
            state: self.state,
            prev: self.prev,
            current: self.current,
            j: self.j,
            word_start: self.word_start,
            delimit_quote: self.delimit_quote,
        }
    }

    fn backtrack(&mut self, snap: BacktrackSnapshot) {
        self.state = snap.state;
        self.prev = snap.prev;
        self.current = snap.current;
        self.j = snap.j;
        self.word_start = snap.word_start;
        self.delimit_quote = snap.delimit_quote;
    }

    fn peek(&self) -> Option<InputChar> {
        let mut char = self.chars.get(self.j).cloned()?;
        if char != b'\\' || matches!(self.state, State::Single) {
            return Some(InputChar {
                char,
                escaped: false,
            });
        }

        match self.state {
            State::Normal => {
                let peeked = self.chars.get(self.j + 1).cloned()?;
                char = peeked;
            }
            State::Single => unreachable!(),
            State::Double => {
                let peeked = self.chars.get(self.j + 1).cloned()?;
                match peeked {
                    b'$' | b'`' | b'"' | b'\\' | b'\n' | b'#' => {
                        char = peeked;
                    }
                    _ => {
                        return Some(InputChar {
                            char,
                            escaped: false,
                        });
                    }
                }
            }
        }
        Some(InputChar {
            char,
            escaped: true,
        })
    }

    fn eat(&mut self) -> Option<InputChar> {
        if let Some(peeked) = self.peek() {
            self.prev = self.current;
            self.current = Some(peeked);
            self.j += 1 + peeked.escaped as usize;
            return Some(peeked);
        }
        return None;
    }

    fn break_word(&mut self, add_delimiter: bool) {
        self.break_word_impl(add_delimiter, false, false)
    }

    fn break_word_operator(&mut self) {
        self.break_word_impl(true, false, true)
    }

    fn break_word_impl(&mut self, add_delimiter: bool, in_normal_space: bool, in_operator: bool) {
        let start = self.word_start;
        let end = self.j;
        if start != end || self.is_immediately_escaped_quote() {
            let token = str::from_utf8(&self.chars[start..end]).unwrap().into();
            match self.state {
                State::Normal => self.tokens.push(Token::Text(token)),
                State::Single => self.tokens.push(Token::SingleQuotedText(token)),
                State::Double => self.tokens.push(Token::DoubleQuotedText(token)),
            }
            if add_delimiter {
                self.tokens.push(Token::Delimit);
            }
        } else if (in_normal_space || in_operator)
            && match self.tokens.last() {
                Some(
                    Token::Var(_)
                    | Token::VarArgv(_)
                    | Token::Text(_)
                    | Token::SingleQuotedText(_)
                    | Token::DoubleQuotedText(_)
                    | Token::BraceBegin
                    | Token::Comma
                    | Token::BraceEnd
                    | Token::CmdSubstEnd
                    | Token::Asterisk,
                ) => true,
                _ => false,
            }
        {
            self.tokens.push(Token::Delimit);
            self.delimit_quote = false;
        }
        self.word_start = self.j;
    }

    fn eat_simple_redirect(&mut self, dir_in: bool) -> RedirectFlags {
        let is_double = self.eat_simple_redirect_operator(dir_in);

        if is_double {
            if dir_in {
                RedirectFlags::leftleft()
            } else {
                RedirectFlags::rightright()
            }
        } else {
            if dir_in {
                RedirectFlags::right()
            } else {
                RedirectFlags::left()
            }
        }
    }

    fn eat_simple_redirect_operator(&mut self, dir_in: bool) -> bool {
        if let Some(peeked) = self.peek() {
            if peeked.escaped {
                return false;
            }
            match peeked.char {
                b'>' => {
                    if !dir_in {
                        self.eat();
                        return true;
                    }
                    return false;
                }
                b'<' => {
                    if dir_in {
                        self.eat();
                        return true;
                    }
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    fn eat_redirect(&mut self, first: InputChar) -> Option<RedirectFlags> {
        let mut flags = RedirectFlags::default();
        match first.char {
            b'0' => flags.stdin = true,
            b'1' => flags.stdout = true,
            b'2' => flags.stderr = true,
            _ => return None,
        }
        let input = self.peek()?;
        match input.char {
            b'>' => {
                self.eat();
                let is_double = self.eat_simple_redirect_operator(false);
                if is_double {
                    flags.append = true
                };
                if let Some(peeked) = self.peek() {
                    if !peeked.escaped && peeked.char == b'&' {
                        self.eat();
                        let peeked2 = self.peek()?;
                        self.eat();
                        match peeked2.char {
                            b'1' => {
                                if !flags.stdout && flags.stderr {
                                    flags.duplicate_out = true;
                                    flags.stdout = true;
                                    flags.stderr = false;
                                } else {
                                    return None;
                                }
                            }
                            b'2' => {
                                if !flags.stderr && flags.stdout {
                                    flags.duplicate_out = true;
                                    flags.stderr = true;
                                    flags.stdout = false;
                                } else {
                                    return None;
                                }
                            }
                            _ => return None,
                        }
                    }
                }
            }
            b'<' => {
                let is_double = self.eat_simple_redirect_operator(true);
                if is_double {
                    flags.append = true
                }
                return Some(flags);
            }
            _ => return None,
        }
        Some(flags)
    }

    fn eat_number_word(&mut self) -> Option<usize> {
        let snap = self.make_snapshot();
        let mut buf = vec![];
        while let Some(result) = self.eat() {
            match result.char {
                b'0'..=b'9' => {
                    buf.push(result.char);
                }
                _ => break,
            }
        }
        if buf.is_empty() {
            self.backtrack(snap);
            return None;
        }
        let result = str::from_utf8(&buf).unwrap().parse::<usize>().ok();
        if result.is_none() {
            self.backtrack(snap);
        }
        result
    }

    fn is_immediately_escaped_quote(&mut self) -> bool {
        matches!(self.state, State::Double)
            && (self
                .current
                .is_some_and(|current| current.escaped && current.char == b'"')
                && (self
                    .prev
                    .is_some_and(|prev| prev.escaped && prev.char == b'"')))
    }

    fn eat_subshell(&mut self, kind: SubShellKind) {
        if let SubShellKind::Dollar = kind {
            self.eat();
        }
        match kind {
            SubShellKind::Normal => self.tokens.push(Token::OpenParen),
            _ => {
                self.tokens.push(Token::CmdSubstBegin);
                if matches!(self.state, State::Double) {
                    self.tokens.push(Token::CmdSubstQuoted);
                }
            }
        }
        let prev_quote_state = self.state;
        let mut sublexer = self.make_sublexer(kind);
        sublexer.lex();
        let j = sublexer.j;
        let word_start = sublexer.word_start;
        let prev = sublexer.prev;
        let current = sublexer.current;
        let delimit_quote = sublexer.delimit_quote;
        drop(sublexer);
        self.j = j;
        self.word_start = word_start;
        self.prev = prev;
        self.current = current;
        self.delimit_quote = delimit_quote;
        self.state = prev_quote_state;
    }

    fn make_sublexer(&mut self, kind: SubShellKind) -> Lexer {
        let sublexer = Lexer {
            chars: self.chars,
            j: self.j,
            word_start: self.word_start,
            state: self.state,
            tokens: self.tokens,
            prev: self.prev,
            current: self.current,
            delimit_quote: self.delimit_quote,
            in_subshell: Some(kind),
        };
        self.state = State::Normal;
        sublexer
    }

    fn eat_var(&mut self) -> Range<usize> {
        let start = self.j;
        let mut i = 0;
        let mut is_int = false;
        while let Some(result) = self.peek() {
            let char = result.char;
            let escaped = result.escaped;
            if i == 0 {
                match char {
                    b'=' => return start..self.j,
                    b'0'..=b'9' => {
                        is_int = true;
                        self.eat();
                        i += 1;
                        continue;
                    }
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' => {}
                    _ => return start..self.j,
                }
            }
            if is_int {
                return start..self.j;
            }
            match char {
                b'{' | b'}' | b';' | b'\'' | b'\"' | b' ' | b'|' | b'&' | b'>' | b',' | b'$' => {
                    return start..self.j;
                }
                _ => {
                    if !escaped
                        && ((matches!(self.in_subshell, Some(SubShellKind::Dollar))
                            && char == b')')
                            || (matches!(self.in_subshell, Some(SubShellKind::Backtick))
                                && char == b'`')
                            || (matches!(self.in_subshell, Some(SubShellKind::Normal))
                                && char == b')'))
                    {
                        return start..self.j;
                    }
                    if let b'0'..=b'9' | b'a'..=b'z' | b'_' = char {
                        self.eat().unwrap();
                    } else {
                        return start..self.j;
                    }
                }
            }
            i += 1;
        }
        start..self.j
    }
}
