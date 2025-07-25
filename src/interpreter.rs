mod run_assigns;
mod run_async;
mod run_atom;
mod run_binary;
mod run_cmd;
mod run_cond_expr;
mod run_if;
mod run_pipeline;
mod run_sub_shell;

use std::{
    marker::Unpin,
    os::unix::process::ExitStatusExt,
    process::{ExitStatus, Stdio},
    sync::Arc,
};

use tokio::{io, sync::Mutex};

use crate::ast;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub async fn run_script(
        &mut self,
        script: &ast::Script,
        stdin: Stdin,
        stdout: Stdout,
        stderr: Stdout,
    ) -> io::Result<ExitStatus> {
        let mut exitstatus = ExitStatus::from_raw(0);
        for stmt in &script.stmts {
            exitstatus = self
                .run_stmt(stmt, stdin.clone(), stdout.clone(), stderr.clone())
                .await?;
        }
        Ok(exitstatus)
    }

    pub async fn run_stmt(
        &mut self,
        stmt: &ast::Stmt,
        stdin: Stdin,
        stdout: Stdout,
        stderr: Stdout,
    ) -> io::Result<ExitStatus> {
        let mut exitstatus = ExitStatus::from_raw(0);
        for expr in &stmt.exprs {
            exitstatus = self
                .run_expr(expr, stdin.clone(), stdout.clone(), stderr.clone())
                .await?;
        }
        Ok(exitstatus)
    }

    pub async fn run_expr(
        &mut self,
        expr: &ast::Expr,
        stdin: Stdin,
        stdout: Stdout,
        stderr: Stdout,
    ) -> io::Result<ExitStatus> {
        match expr {
            ast::Expr::Assign(assigns) => self.run_assigns(assigns).await,
            ast::Expr::Binary(binary) => self.run_binary(binary).await,
            ast::Expr::Pipeline(pipeline) => {
                self.run_pipeline(pipeline, stdin, stdout, stderr).await
            }
            ast::Expr::Cmd(cmd) => self.run_cmd(cmd, stdin, stdout, stderr).await,
            ast::Expr::SubShell(sub_shell) => self.run_sub_shell(sub_shell).await,
            ast::Expr::If(if_) => self.run_if(if_).await,
            ast::Expr::CondExpr(cond_expr) => self.run_cond_expr(cond_expr).await,
            ast::Expr::Async(expr) => self.run_async(expr).await,
        }
    }
}

#[derive(Clone)]
pub enum Stdin {
    Inherit,
    Pipe(Arc<Mutex<dyn io::AsyncRead + Send + Unpin>>),
}

#[derive(Clone)]
pub enum Stdout {
    Inherit,
    Pipe(Arc<Mutex<dyn io::AsyncWrite + Send + Unpin>>),
}

impl From<&Stdin> for Stdio {
    fn from(stdin: &Stdin) -> Self {
        match stdin {
            Stdin::Inherit => Stdio::inherit(),
            Stdin::Pipe(_) => Stdio::piped(),
        }
    }
}

impl From<&Stdout> for Stdio {
    fn from(stdout: &Stdout) -> Self {
        match stdout {
            Stdout::Inherit => Stdio::inherit(),
            Stdout::Pipe(_) => Stdio::piped(),
        }
    }
}
