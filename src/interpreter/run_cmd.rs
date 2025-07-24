use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_cmd(&mut self, cmd: &ast::Cmd) {
        eprintln!("{:#?}", cmd.name_and_args);
        todo!()
    }
}
