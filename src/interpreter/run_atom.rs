use crate::{
    ast,
    interpreter::{run_script, Stdin, Stdout},
    stringpool::StringPool,
};
use std::sync::Arc;
use tokio::{io, sync::Mutex};

pub async fn run_atom(atom: &ast::Atom, out: &mut StringPool) -> io::Result<()> {
    match atom {
        ast::Atom::Simple(simple_atom) => run_simple_atom(simple_atom, out).await,
        ast::Atom::CompoundAtom(compound_atom) => run_compound_atom(compound_atom, out).await,
    }
}

async fn run_simple_atom(simple_atom: &ast::SimpleAtom, out: &mut StringPool) -> io::Result<()> {
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
        ast::SimpleAtom::CmdSubst { script, quoted } => run_cmd_subst(script, *quoted, out).await,
    }
}

async fn run_compound_atom(
    compound_atom: &ast::CompoundAtom,
    out: &mut StringPool,
) -> io::Result<()> {
    todo!()
}

async fn run_cmd_subst(script: &ast::Script, quoted: bool, out: &mut StringPool) -> io::Result<()> {
    let stdout = Arc::new(Mutex::new(vec![]));
    Box::pin(run_script(
        script,
        Stdin::Inherit,
        Stdout::Pipe(stdout.clone()),
        Stdout::Inherit,
    ))
    .await?;
    let stdout = stdout.lock().await;
    let stdout = stdout.as_slice();
    if quoted {
        out.push_str(stdout);
    } else {
        word_splitting(stdout, out);
    }
    Ok(())
}

fn word_splitting(text: &[u8], out: &mut StringPool) {
    for word in text.split(|b| *b == b' ' || *b == b'\n' || *b == b'\t') {
        out.push_str(word);
    }
}
