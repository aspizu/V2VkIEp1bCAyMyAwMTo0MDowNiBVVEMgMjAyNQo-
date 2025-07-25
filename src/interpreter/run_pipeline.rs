use std::process::ExitStatus;

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
        let mut prev = None;
        for (i, item) in pipeline.items.iter().rev().enumerate() {
            let (reader, writer) = io::simplex(1024);

            let stdout = if i == 0 {
                Stdout::Inherit
            } else {
                Stdout::Pipe(writer)
            };
            let stdin = Stdin::Pipe(prev.take().unwrap());
            match item {
                ast::PipelineItem::Cmd(cmd) => {
                    futures.push(Box::pin(self.run_cmd(cmd, &mut stdin, &mut stdout, stderr)));
                }
                ast::PipelineItem::Assigns(assigns) => todo!(),
                ast::PipelineItem::SubShell(sub_shell) => todo!(),
                ast::PipelineItem::If(_) => todo!(),
                ast::PipelineItem::CondExpr(cond_expr) => todo!(),
            }
            prev.insert(reader);
        }
        todo!()
    }
}
