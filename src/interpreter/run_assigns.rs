use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_assigns(&mut self, assigns: &[ast::Assign]) -> io::Result<ExitStatus> {
        todo!()
    }
}
