use std::{process::ExitStatus, sync::Arc};

use futures::future::{join_all, BoxFuture};
use tokio::{io, sync::Mutex};

use super::Interpreter;
use crate::{
    ast,
    interpreter::{Stdin, Stdout},
};

impl Interpreter {
    pub async fn run_pipeline(
        &mut self,
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
                    futures.push(Box::pin(self.run_cmd(cmd, stdin, stdout, stderr)));
                }
                ast::PipelineItem::Assigns(assigns) => todo!(),
                ast::PipelineItem::SubShell(sub_shell) => todo!(),
                ast::PipelineItem::If(_) => todo!(),
                ast::PipelineItem::CondExpr(cond_expr) => todo!(),
            }
            prev.insert(reader);
        }
        for result in join_all(futures).await {
            result?;
        }
        todo!()
    }
}
