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
            Token::Pipe => format!("`|`"),
            Token::DoublePipe => format!("`||`"),
            Token::Ampersand => format!("`&`"),
            Token::DoubleAmpersand => format!("`&&`"),
            Token::Redirect(redirect_flags) => format!("{:?}", redirect_flags),
            Token::Dollar => format!("`$`"),
            Token::Asterisk => format!("`*`"),
            Token::DoubleAsterisk => format!("`**`"),
            Token::Eq => format!("`=`"),
            Token::Semicolon => format!("`;`"),
            Token::Newline => format!("`\\n`"),
            Token::BraceBegin => format!("`{{`"),
            Token::Comma => format!("`,`"),
            Token::BraceEnd => format!("`}}`"),
            Token::CmdSubstBegin => format!("CmdSubstBegin"),
            Token::CmdSubstQuoted => format!("CmdSubstQuoted"),
            Token::CmdSubstEnd => format!("CmdSubstEnd"),
            Token::OpenParen => format!("`(`"),
            Token::CloseParen => format!("`)`"),
            Token::Var(range) => getstr(arena, range),
            Token::VarArgv(n) => format!("$argv[{}]", n),
            Token::Text(range) => getstr(arena, range),
            Token::SingleQuotedText(range) => getstr(arena, range),
            Token::DoubleQuotedText(range) => getstr(arena, range),
            Token::PyObject(_) => format!("PyObject"),
            Token::DoubleBracketOpen => format!("`[[`"),
            Token::DoubleBracketClose => format!("`]]`"),
            Token::Delimit => format!("Delimit"),
            Token::Eof => format!("EOF"),
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
