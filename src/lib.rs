mod ast;
mod interpreter;
mod lexer;
mod parser;
mod templatelib;
mod tokens;

use pyo3::prelude::*;

use crate::{lexer::Lexer, parser::Parser, tokens::stringify_tokens};

#[pyfunction]
fn _lex_command<'py>(py: Python<'py>, command: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let command = command.extract::<String>()?;
    let bytes = command.as_bytes();
    let mut tokens = vec![];
    let mut arena = vec![];
    let mut lexer = Lexer::new(bytes, &mut tokens, &mut arena);
    lexer.lex();
    let dbg = stringify_tokens(&tokens, &arena);
    let result = dbg.into_pyobject(py)?;
    Ok(result.into_any())
}

#[pyfunction]
fn _parse_command<'py>(py: Python<'py>, command: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let command = command.extract::<String>()?;
    let bytes = command.as_bytes();
    let mut tokens = vec![];
    let mut arena = vec![];
    let mut lexer = Lexer::new(bytes, &mut tokens, &mut arena);
    lexer.lex();
    let mut parser = Parser::new(&tokens, &arena);
    let script = parser.parse();
    let dbg = format!("{:?}", script);
    let result = dbg.into_pyobject(py)?;
    Ok(result.into_any())
}

#[pymodule]
fn shl(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_lex_command, m)?)?;
    m.add_function(wrap_pyfunction!(_parse_command, m)?)?;
    Ok(())
}
