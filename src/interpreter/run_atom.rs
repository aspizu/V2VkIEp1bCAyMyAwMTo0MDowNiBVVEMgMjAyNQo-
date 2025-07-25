use arcstr::ArcStr;
use tokio::io;

use crate::{
    ast,
    interpreter::{Interpreter, Stdin, Stdout},
    stringpool::StringPool,
};

impl Interpreter {
    pub async fn run_atom(&mut self, atom: &ast::Atom, out: &mut StringPool) -> io::Result<()> {
        match atom {
            ast::Atom::Simple(simple_atom) => self.run_simple_atom(simple_atom, out).await,
            ast::Atom::CompoundAtom(compound_atom) => {
                self.run_compound_atom(compound_atom, out).await
            }
        }
    }

    async fn run_simple_atom(
        &mut self,
        simple_atom: &ast::SimpleAtom,
        out: &mut StringPool,
    ) -> io::Result<()> {
        match simple_atom {
            ast::SimpleAtom::Var(var) => todo!(),
            ast::SimpleAtom::VarArgv(_) => todo!(),
            ast::SimpleAtom::Text(text) => {
                out.push(text.clone());
                Ok(())
            }
            ast::SimpleAtom::Asterisk => todo!(),
            ast::SimpleAtom::DoubleAsterisk => todo!(),
            ast::SimpleAtom::BraceBegin => todo!(),
            ast::SimpleAtom::BraceEnd => todo!(),
            ast::SimpleAtom::Comma => todo!(),
            ast::SimpleAtom::Tilde => todo!(),
            ast::SimpleAtom::CmdSubst { script, quoted } => {
                self.run_cmd_subst(script, *quoted, out).await
            }
        }
    }

    async fn run_compound_atom(
        &mut self,
        compound_atom: &ast::CompoundAtom,
        out: &mut StringPool,
    ) -> io::Result<()> {
        todo!()
    }

    async fn run_cmd_subst(
        &mut self,
        script: &ast::Script,
        quoted: bool,
        out: &mut StringPool,
    ) -> io::Result<()> {
        let mut stdout: Vec<u8> = vec![];
        Box::pin(self.run_script(
            script,
            &mut Stdin::<&[u8]>::Inherit,
            &mut Stdout::Pipe(&mut stdout),
            &mut Stdout::<Vec<u8>>::Inherit,
        ))
        .await?;
        if quoted {
            out.push_str(&stdout);
        } else {
            word_splitting(&stdout, out);
        }
        Ok(())
    }
}

fn word_splitting(text: &[u8], out: &mut StringPool) {
    for word in text.split(|b| *b == b' ' || *b == b'\n' || *b == b'\t') {
        out.push_str(word);
    }
}
