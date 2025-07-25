use bytes::Bytes;

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
    Var(Bytes),
    VarArgv(u8),
    Text(Bytes),
    SingleQuotedText(Bytes),
    DoubleQuotedText(Bytes),
    PyObject(usize),
    DoubleBracketOpen,
    DoubleBracketClose,
    Delimit,
    Eof,
}
