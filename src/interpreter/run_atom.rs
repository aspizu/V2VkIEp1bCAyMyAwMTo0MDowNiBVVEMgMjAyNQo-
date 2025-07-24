
use arcstr::ArcStr;
use tokio::io;

use crate::{
    ast,
    interpreter::{Interpreter, Stdin, Stdout},
};

impl Interpreter {
    pub async fn run_atom(&mut self, atom: &ast::Atom) -> io::Result<ArcStr> {
        match atom {
            ast::Atom::Simple(simple_atom) => self.run_simple_atom(simple_atom).await,
            ast::Atom::CompoundAtom(compound_atom) => self.run_compound_atom(compound_atom).await,
        }
    }

    async fn run_simple_atom(&mut self, simple_atom: &ast::SimpleAtom) -> io::Result<ArcStr> {
        match simple_atom {
            ast::SimpleAtom::Var(arc_str) => todo!(),
            ast::SimpleAtom::VarArgv(_) => todo!(),
            ast::SimpleAtom::Text(arc_str) => Ok(arc_str.clone()),
            ast::SimpleAtom::Asterisk => todo!(),
            ast::SimpleAtom::DoubleAsterisk => todo!(),
            ast::SimpleAtom::BraceBegin => todo!(),
            ast::SimpleAtom::BraceEnd => todo!(),
            ast::SimpleAtom::Comma => todo!(),
            ast::SimpleAtom::Tilde => todo!(),
            ast::SimpleAtom::CmdSubst { script, quoted } => {
                self.run_cmd_subst(script, *quoted).await
            }
        }
    }

    async fn run_compound_atom(&mut self, compound_atom: &ast::CompoundAtom) -> io::Result<ArcStr> {
        todo!()
    }

    async fn run_cmd_subst(&mut self, script: &ast::Script, quoted: bool) -> io::Result<ArcStr> {
        let mut stdout: Vec<u8> = vec![];
        Box::pin(self.run_script(
            script,
            &mut Stdin::<&[u8]>::Inherit,
            &mut Stdout::Pipe(&mut stdout),
            &mut Stdout::<Vec<u8>>::Inherit,
        ))
        .await?;
        Ok(str::from_utf8(&stdout)
            .unwrap()
            .trim_end_matches('\n')
            .into())
    }
}
