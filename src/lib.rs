mod ast;
mod interpreter;
mod lexer;
mod parser;
mod templatelib;
mod tokens;

// use pyo3::prelude::*;

// #[pyfunction]
// fn _execute_command<'py>(
//     py: Python<'py>,
//     command: Bound<'py, PyAny>,
// ) -> PyResult<Bound<'py, PyAny>> {
//     let tokens = lex(py, command)?;
//     let fut = async move {
//         let mut parser = ShellParser::new(tokens);
//         let ast = parser
//             .parse()
//             .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
//         interpreter::interpret(ast.clone()).await;
//         Ok(format!("{}", ast))
//     };
//     pyo3_async_runtimes::async_std::future_into_py(py, fut)
// }

// #[pymodule]
// fn shl(m: &Bound<PyModule>) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(_execute_command, m)?)?;
//     Ok(())
// }
