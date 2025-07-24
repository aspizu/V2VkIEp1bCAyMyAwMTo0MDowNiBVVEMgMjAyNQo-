use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_sub_shell(&mut self, sub_shell: &ast::SubShell) -> io::Result<ExitStatus> {
        todo!()
    }
}
