use std::{ops::Range, str};

use crate::ast::RedirectFlags;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Pipe,
    DoublePipe,
    Ampersand,
    DoubleAmpersand,
    Redirect(RedirectFlags),
    Dollar,
    Asterisk,
    DoubleAsterisk,
    Eq,
    Semicolon,
    Newline,
    BraceBegin,
    Comma,
    BraceEnd,
    CmdSubstBegin,
    CmdSubstQuoted,
    CmdSubstEnd,
    OpenParen,
    CloseParen,
    Var(Range<usize>),
    VarArgv(u8),
    Text(Range<usize>),
    SingleQuotedText(Range<usize>),
    DoubleQuotedText(Range<usize>),
    PyObject(usize),
    DoubleBracketOpen,
    DoubleBracketClose,
    Delimit,
    Eof,
}

fn getstr(arena: &[u8], range: &Range<usize>) -> String {
    let s = str::from_utf8(&arena[range.clone()]).unwrap();
    format!("```{}```", s)
}

impl Token {
    pub fn stringify(&self, arena: &[u8]) -> String {
        match self {
            Token::Pipe => "`|`".to_string(),
            Token::DoublePipe => "`||`".to_string(),
            Token::Ampersand => "`&`".to_string(),
            Token::DoubleAmpersand => "`&&`".to_string(),
            Token::Redirect(redirect_flags) => format!("{:?}", redirect_flags),
            Token::Dollar => "`$`".to_string(),
            Token::Asterisk => "`*`".to_string(),
            Token::DoubleAsterisk => "`**`".to_string(),
            Token::Eq => "`=`".to_string(),
            Token::Semicolon => "`;`".to_string(),
            Token::Newline => "`\\n`".to_string(),
            Token::BraceBegin => "`{`".to_string(),
            Token::Comma => "`,`".to_string(),
            Token::BraceEnd => "`}`".to_string(),
            Token::CmdSubstBegin => "CmdSubstBegin".to_string(),
            Token::CmdSubstQuoted => "CmdSubstQuoted".to_string(),
            Token::CmdSubstEnd => "CmdSubstEnd".to_string(),
            Token::OpenParen => "`(`".to_string(),
            Token::CloseParen => "`)`".to_string(),
            Token::Var(range) => getstr(arena, range),
            Token::VarArgv(n) => format!("$argv[{}]", n),
            Token::Text(range) => getstr(arena, range),
            Token::SingleQuotedText(range) => getstr(arena, range),
            Token::DoubleQuotedText(range) => getstr(arena, range),
            Token::PyObject(_) => "PyObject".to_string(),
            Token::DoubleBracketOpen => "`[[`".to_string(),
            Token::DoubleBracketClose => "`]]`".to_string(),
            Token::Delimit => "Delimit".to_string(),
            Token::Eof => "EOF".to_string(),
        }
    }
}

pub fn stringify_tokens(tokens: &[Token], arena: &[u8]) -> String {
    tokens
        .iter()
        .map(|token| token.stringify(arena))
        .collect::<Vec<_>>()
        .join("\n")
}
