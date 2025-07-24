use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_async(&mut self, expr: &ast::Expr) -> io::Result<ExitStatus> {
        todo!()
    }
}
