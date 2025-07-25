use crate::{
    ast,
    interpreter::{run_atom::run_atom, Stdin, Stdout},
    stringpool::StringPool,
};
use futures::future::{join_all, BoxFuture};
use std::{future::Future, ops::DerefMut, process::ExitStatus};
use tokio::{
    io::{self, AsyncWriteExt},
    process::Command,
};

pub fn run_cmd<'a, 'b>(
    cmd: &'b ast::Cmd,
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stdout,
) -> impl Future<Output = io::Result<ExitStatus>> + Send + use<'a, 'b> {
    async move {
        let mut args = StringPool::new();
        for arg in &cmd.name_and_args {
            run_atom(arg, &mut args).await?;
        }
        let mut child = Command::new(str::from_utf8(&args.get_strings()[0]).unwrap())
            .args(
                args.get_strings()[1..]
                    .iter()
                    .map(|s| str::from_utf8(s).unwrap()),
            )
            .stdin(&stdin)
            .stdout(&stdout)
            .stderr(&stderr)
            .spawn()
            .unwrap();
        let mut stdin = if let Stdin::Pipe(stdin) = &stdin {
            Some(stdin.lock().await)
        } else {
            None
        };
        let mut stdout = if let Stdout::Pipe(stdout) = &stdout {
            Some(stdout.lock().await)
        } else {
            None
        };
        let mut stderr = if let Stdout::Pipe(stderr) = &stderr {
            Some(stderr.lock().await)
        } else {
            None
        };
        let bump = bumpalo::Bump::new();
        let mut futures: Vec<BoxFuture<io::Result<u64>>> = vec![];
        if let Some(stdin) = &mut stdin {
            let child_stdin = bump.alloc(child.stdin.take().unwrap());
            let stdin = stdin.deref_mut();
            futures.push(Box::pin(io::copy(stdin, child_stdin)));
        }
        if let Some(stdout) = &mut stdout {
            let child_stdout = bump.alloc(child.stdout.take().unwrap());
            let stdout = stdout.deref_mut();
            futures.push(Box::pin(io::copy(child_stdout, stdout)));
        }
        if let Some(stderr) = &mut stderr {
            let child_stderr = bump.alloc(child.stderr.take().unwrap());
            let stderr = stderr.deref_mut();
            futures.push(Box::pin(io::copy(child_stderr, stderr)));
        }
        for result in join_all(futures).await {
            result?;
        }
        let success = child.wait().await?;
        if let Some(mut stdout) = stdout {
            stdout.shutdown().await?;
        }
        if let Some(mut stderr) = stderr {
            stderr.shutdown().await?;
        }
        Ok(success)
    }
}
