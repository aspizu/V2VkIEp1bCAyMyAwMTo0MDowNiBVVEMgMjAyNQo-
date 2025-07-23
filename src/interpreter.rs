mod run_assigns;
mod run_async;
mod run_binary;
mod run_cmd;
mod run_cond_expr;
mod run_if;
mod run_pipeline;
mod run_sub_shell;

use crate::ast;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub async fn run_script(&mut self, script: &ast::Script) {
        for stmt in &script.stmts {
            self.run_stmt(stmt).await;
        }
    }

    pub async fn run_stmt(&mut self, stmt: &ast::Stmt) {
        for expr in &stmt.exprs {
            self.run_expr(expr).await;
        }
    }

    pub async fn run_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::Assign(assigns) => self.run_assigns(assigns).await,
            ast::Expr::Binary(binary) => self.run_binary(binary).await,
            ast::Expr::Pipeline(pipeline) => self.run_pipeline(pipeline).await,
            ast::Expr::Cmd(cmd) => self.run_cmd(cmd).await,
            ast::Expr::SubShell(sub_shell) => self.run_sub_shell(sub_shell).await,
            ast::Expr::If(if_) => self.run_if(if_).await,
            ast::Expr::CondExpr(cond_expr) => self.run_cond_expr(cond_expr).await,
            ast::Expr::Async(expr) => self.run_async(expr).await,
        }
    }
}
