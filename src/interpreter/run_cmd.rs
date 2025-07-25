use std::{marker::Unpin, os::unix::process::ExitStatusExt, process::ExitStatus};

use super::Interpreter;
use crate::{
    ast,
    interpreter::{Stdin, Stdout},
    stringpool::StringPool,
};
use futures::future::{join_all, BoxFuture};
use tokio::{io, process::Command};

impl Interpreter {
    pub async fn run_cmd(
        &mut self,
        cmd: &ast::Cmd,
        stdin: &mut Stdin<impl io::AsyncRead + Unpin + Send>,
        stdout: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
        stderr: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
    ) -> io::Result<ExitStatus> {
        let mut args = StringPool::new();
        for arg in &cmd.name_and_args {
            self.run_atom(arg, &mut args).await?;
        }
        let mut child = Command::new(str::from_utf8(&args.get_strings()[0]).unwrap())
            .args(
                args.get_strings()[1..]
                    .iter()
                    .map(|s| str::from_utf8(s).unwrap()),
            )
            .stdin(&mut *stdin)
            .stdout(&mut *stdout)
            .stderr(&mut *stderr)
            .spawn()
            .unwrap();
        let bump = bumpalo::Bump::new();
        let mut futures: Vec<BoxFuture<io::Result<u64>>> = vec![];
        if let Stdin::Pipe(stdin) = stdin {
            let child_stdin = bump.alloc(child.stdin.take().unwrap());
            futures.push(Box::pin(io::copy(stdin, child_stdin)));
        }
        if let Stdout::Pipe(stdout) = stdout {
            let child_stdout = bump.alloc(child.stdout.take().unwrap());
            futures.push(Box::pin(io::copy(child_stdout, stdout)));
        }
        if let Stdout::Pipe(stderr) = stderr {
            let child_stderr = bump.alloc(child.stderr.take().unwrap());
            futures.push(Box::pin(io::copy(child_stderr, stderr)));
        }
        for result in join_all(futures).await {
            result?;
        }
        Ok(ExitStatus::from_raw(child.wait().await?.code().unwrap()))
    }
}
