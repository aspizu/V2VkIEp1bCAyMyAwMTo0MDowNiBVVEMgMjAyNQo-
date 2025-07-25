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
};

use tokio::io;

use crate::ast;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub async fn run_script(
        &mut self,
        script: &ast::Script,
        stdin: &mut Stdin<impl io::AsyncRead + Unpin + Send>,
        stdout: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
        stderr: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
    ) -> io::Result<ExitStatus> {
        let mut exitstatus = ExitStatus::from_raw(0);
        for stmt in &script.stmts {
            exitstatus = self.run_stmt(stmt, stdin, stdout, stderr).await?;
        }
        Ok(exitstatus)
    }

    pub async fn run_stmt(
        &mut self,
        stmt: &ast::Stmt,
        stdin: &mut Stdin<impl io::AsyncRead + Unpin + Send>,
        stdout: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
        stderr: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
    ) -> io::Result<ExitStatus> {
        let mut exitstatus = ExitStatus::from_raw(0);
        for expr in &stmt.exprs {
            exitstatus = self.run_expr(expr, stdin, stdout, stderr).await?;
        }
        Ok(exitstatus)
    }

    pub async fn run_expr(
        &mut self,
        expr: &ast::Expr,
        stdin: &mut Stdin<impl io::AsyncRead + Unpin + Send>,
        stdout: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
        stderr: &mut Stdout<impl io::AsyncWrite + Unpin + Send>,
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

pub enum Stdin<T>
where
    T: io::AsyncRead + Unpin,
{
    Inherit,
    Pipe(T),
}

pub enum Stdout<T>
where
    T: io::AsyncWrite + Unpin,
{
    Inherit,
    Pipe(T),
}

impl<T> From<&mut Stdin<T>> for Stdio
where
    T: io::AsyncRead + Unpin,
{
    fn from(stdin: &mut Stdin<T>) -> Self {
        match stdin {
            Stdin::Inherit => Stdio::inherit(),
            Stdin::Pipe(_) => Stdio::piped(),
        }
    }
}

impl<T> From<&mut Stdout<T>> for Stdio
where
    T: io::AsyncWrite + Unpin,
{
    fn from(stdout: &mut Stdout<T>) -> Self {
        match stdout {
            Stdout::Inherit => Stdio::inherit(),
            Stdout::Pipe(_) => Stdio::piped(),
        }
    }
}

impl<T: io::AsyncRead + Unpin + Send> Stdin<T> {
    pub fn as_dyn_reader(&mut self) -> Box<dyn io::AsyncRead + Send + Unpin + '_> {
        match self {
            Stdin::Pipe(inner) => Box::new(inner),
            Stdin::Inherit => Box::new(tokio::io::empty()),
        }
    }
}

impl<T: io::AsyncWrite + Unpin + Send> Stdout<T> {
    pub fn as_dyn_writer(&mut self) -> Box<dyn io::AsyncWrite + Send + Unpin + '_> {
        match self {
            Stdout::Pipe(inner) => Box::new(inner),
            Stdout::Inherit => Box::new(tokio::io::empty()),
        }
    }
}
