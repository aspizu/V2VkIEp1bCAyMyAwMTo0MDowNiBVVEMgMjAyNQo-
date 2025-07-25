#![feature(slice_split_once)]

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod stringpool;
mod templatelib;
mod tokens;

use pyo3::prelude::*;

use crate::{
    interpreter::{Interpreter, Stdin, Stdout},
    lexer::{Lexer, PLACEHOLDER},
    parser::Parser,
};

fn split_template<'py>(command: Bound<'py, PyAny>) -> PyResult<(Vec<Bound<'py, PyAny>>, Vec<u8>)> {
    let mut pyobjects = vec![];
    let mut bytes = vec![];
    for part in command.try_iter()? {
        let part = part?;
        if let Ok(text) = part.extract::<&str>() {
            bytes.extend_from_slice(text.as_bytes());
        } else {
            let value = part.getattr("value")?;
            pyobjects.push(value);
            bytes.push(PLACEHOLDER);
        }
    }
    Ok((pyobjects, bytes))
}

// #[pyfunction]
// fn _lex_command<'py>(py: Python<'py>, command: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
//     let (pyobjects, bytes) = split_template(command)?;
//     let mut tokens = vec![];
//     let mut arena = vec![];
//     let mut lexer = Lexer::new(&bytes, &mut tokens, &mut arena, &pyobjects);
//     lexer.lex()?;
//     let dbg = stringify_tokens(&tokens, &arena);
//     let result = dbg.into_pyobject(py)?;
//     Ok(result.into_any())
// }

#[pyfunction]
fn _parse_command<'py>(py: Python<'py>, command: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let (pyobjects, bytes) = split_template(command)?;
    let mut tokens = vec![];
    let mut arena = vec![];
    let mut lexer = Lexer::new(&bytes, &mut tokens, &mut arena, &pyobjects);
    lexer.lex()?;
    let mut parser = Parser::new(&tokens, &arena);
    let script = parser.parse();
    let dbg = format!("{:?}", script);
    let result = dbg.into_pyobject(py)?;
    Ok(result.into_any())
}

#[pyfunction]
fn _execute_command<'py>(
    py: Python<'py>,
    command: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let (pyobjects, bytes) = split_template(command)?;
    let mut tokens = vec![];
    let mut arena = vec![];
    let mut lexer = Lexer::new(&bytes, &mut tokens, &mut arena, &pyobjects);
    lexer.lex()?;
    let mut parser = Parser::new(&tokens, &arena);
    let script = parser.parse();
    let mut interpreter = Interpreter::new();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        interpreter
            .run_script(&script, Stdin::Inherit, Stdout::Inherit, Stdout::Inherit)
            .await?;
        let dbg = format!("{:?}", script);
        Python::with_gil(|py| Ok(dbg.into_pyobject(py)?.into_any().unbind()))
    })
}

#[pymodule]
fn shl(m: &Bound<PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(_lex_command, m)?)?;
    m.add_function(wrap_pyfunction!(_parse_command, m)?)?;
    m.add_function(wrap_pyfunction!(_execute_command, m)?)?;
    Ok(())
}
