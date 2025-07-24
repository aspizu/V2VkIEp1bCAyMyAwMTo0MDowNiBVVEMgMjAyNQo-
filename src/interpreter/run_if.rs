use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_if(&mut self, if_: &ast::If) -> io::Result<ExitStatus> {
        todo!()
    }
}
