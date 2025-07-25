use crate::ast;
use std::process::ExitStatus;
use tokio::io;

pub async fn run_async(expr: &ast::Expr) -> io::Result<ExitStatus> {
    todo!()
}
