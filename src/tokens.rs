use arcstr::ArcStr;

use crate::ast::RedirectFlags;

#[derive(Clone, Eq, PartialEq)]
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
    Var(ArcStr),
    VarArgv(u8),
    Text(ArcStr),
    SingleQuotedText(ArcStr),
    DoubleQuotedText(ArcStr),
    PyObject(()),
    DoubleBracketOpen,
    DoubleBracketClose,
    Delimit,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Pipe => write!(f, "|"),
            Token::DoublePipe => write!(f, "||"),
            Token::Ampersand => write!(f, "&"),
            Token::DoubleAmpersand => write!(f, "&&"),
            Token::Redirect(_) => write!(f, ">"),
            Token::Dollar => write!(f, "$"),
            Token::Asterisk => write!(f, "*"),
            Token::DoubleAsterisk => write!(f, "**"),
            Token::Eq => write!(f, "="),
            Token::Semicolon => write!(f, ";"),
            Token::Newline => write!(f, "\\n"),
            Token::BraceBegin => write!(f, "{{"),
            Token::Comma => write!(f, ","),
            Token::BraceEnd => write!(f, "}}"),
            Token::CmdSubstBegin => write!(f, "$("),
            Token::CmdSubstQuoted => write!(f, "CmdSubstQuoted"),
            Token::CmdSubstEnd => write!(f, ")"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Var(arc_str) => write!(f, "{arc_str}"),
            Token::VarArgv(n) => write!(f, "argv[{n}]"),
            Token::Text(arc_str) => write!(f, "{arc_str}"),
            Token::SingleQuotedText(arc_str) => write!(f, "{arc_str}"),
            Token::DoubleQuotedText(arc_str) => write!(f, "{arc_str}"),
            Token::PyObject(_) => write!(f, "PyObject"),
            Token::DoubleBracketOpen => write!(f, "[["),
            Token::DoubleBracketClose => write!(f, "]]"),
            Token::Delimit => write!(f, "DELMIT"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}
