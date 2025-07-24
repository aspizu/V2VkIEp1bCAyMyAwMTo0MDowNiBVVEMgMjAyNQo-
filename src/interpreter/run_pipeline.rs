use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_pipeline(&mut self, pipeline: &ast::Pipeline) -> io::Result<ExitStatus> {
        todo!()
    }
}
