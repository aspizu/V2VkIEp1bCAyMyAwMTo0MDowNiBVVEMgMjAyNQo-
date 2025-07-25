use crate::ast;
use std::process::ExitStatus;
use tokio::io;

pub async fn run_if(if_: &ast::If) -> io::Result<ExitStatus> {
    todo!()
}
