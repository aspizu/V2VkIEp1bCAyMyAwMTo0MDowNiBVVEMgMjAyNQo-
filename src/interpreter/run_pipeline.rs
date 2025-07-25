use crate::{
    ast,
    interpreter::{run_cmd::run_cmd, Stdin, Stdout},
};
use futures::future::{join_all, BoxFuture};
use std::{process::ExitStatus, sync::Arc};
use tokio::{io, sync::Mutex};

pub async fn run_pipeline(
    pipeline: &ast::Pipeline,
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stdout,
) -> io::Result<ExitStatus> {
    let bump = bumpalo::Bump::new();
    let mut futures: Vec<BoxFuture<io::Result<ExitStatus>>> = vec![];
    let mut prev = None;
    for (i, item) in pipeline.items.iter().rev().enumerate() {
        let (reader, writer) = io::simplex(1024);
        let stdout = if i == pipeline.items.len() - 1 {
            Stdout::Inherit
        } else {
            Stdout::Pipe(Arc::new(Mutex::new(writer)))
        };
        let stdin = if let Some(prev) = prev.take() {
            Stdin::Pipe(Arc::new(Mutex::new(prev)))
        } else {
            Stdin::Inherit
        };
        match item {
            ast::PipelineItem::Cmd(cmd) => {
                futures.push(Box::pin(run_cmd(cmd, stdin, stdout, stderr.clone())));
            }
            ast::PipelineItem::Assigns(assigns) => todo!(),
            ast::PipelineItem::SubShell(sub_shell) => todo!(),
            ast::PipelineItem::If(_) => todo!(),
            ast::PipelineItem::CondExpr(cond_expr) => todo!(),
        }
        prev = Some(reader);
    }
    let mut exitcode = None;
    for result in join_all(futures).await {
        exitcode = Some(result?);
    }
    Ok(exitcode.unwrap_or_default())
}
