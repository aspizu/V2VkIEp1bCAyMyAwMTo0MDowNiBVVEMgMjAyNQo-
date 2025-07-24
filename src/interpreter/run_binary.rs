use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_binary(&mut self, binary: &ast::Binary) -> io::Result<ExitStatus> {
        todo!()
    }
}
