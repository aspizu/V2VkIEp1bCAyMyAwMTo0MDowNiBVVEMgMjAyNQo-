use std::process::ExitStatus;

use tokio::io;

use super::Interpreter;
use crate::ast;

impl Interpreter {
    pub async fn run_cmd(&mut self, cmd: &ast::Cmd) -> io::Result<ExitStatus> {
        let mut args = vec![];
        for arg in &cmd.name_and_args {
            args.push(self.run_atom(arg).await?);
        }
        let mut cmd = async_process::Command::new(args[0].as_str());
        cmd.args(args[1..].iter().map(|s| s.as_str()));
        let mut child = cmd.spawn().unwrap();
        child.status().await
    }
}
