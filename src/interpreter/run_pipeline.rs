use std::{io::Read, process::ExitStatus};

use futures::future::BoxFuture;
use tokio::io;

use super::Interpreter;
use crate::{
    ast,
    interpreter::{Stdin, Stdout},
};

impl Interpreter {
    pub async fn run_pipeline(
        &mut self,
        pipeline: &ast::Pipeline,
        stdin: &mut Stdin<impl io::AsyncRead + Unpin + Send>,
        stdout: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
        stderr: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
    ) -> io::Result<ExitStatus> {
        let bump = bumpalo::Bump::new();
        let mut futures: Vec<BoxFuture<io::Result<ExitStatus>>> = vec![];
        let mut pipe = None;
        for (i, item) in pipeline.items.iter().rev().enumerate() {
            let stdout: Stdout<Vec<u8>> = if i == 0 {
                Stdout::Inherit
            } else {
                Stdout::Pipe(pipe.take().unwrap())
            };
            let stdin = vec![];
            match item {
                ast::PipelineItem::Cmd(cmd) => {
                    futures.push(Box::pin(self.run_cmd(
                        cmd,
                        &mut Stdin::Pipe(stdin.bytes()),
                        &mut stdout,
                        stderr,
                    )));
                }
                ast::PipelineItem::Assigns(assigns) => todo!(),
                ast::PipelineItem::SubShell(sub_shell) => todo!(),
                ast::PipelineItem::If(_) => todo!(),
                ast::PipelineItem::CondExpr(cond_expr) => todo!(),
            }
            pipe.insert(stdin);
        }
        todo!()
    }
}
