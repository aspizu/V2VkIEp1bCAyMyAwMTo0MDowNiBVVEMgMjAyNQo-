use std::process::ExitStatus;

use tokio::io;

use crate::ast;

pub async fn run_sub_shell(sub_shell: &ast::SubShell) -> io::Result<ExitStatus> {
    todo!()
}
