use arcstr::ArcStr;
use tokio::io;

use crate::{ast, interpreter::Interpreter};

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
            ast::SimpleAtom::CmdSubst { script, quoted } => todo!(),
        }
    }

    async fn run_compound_atom(&mut self, compound_atom: &ast::CompoundAtom) -> io::Result<ArcStr> {
        todo!()
    }
}
