use arcstr::ArcStr;

#[derive(Debug, Clone)]
pub struct Script {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Vec<Assign>),
    Binary(Box<Binary>),
    Pipeline(Pipeline),
    Cmd(Cmd),
    SubShell(SubShell),
    If(If),
    CondExpr(CondExpr),
    Async(Box<Expr>),
}

impl Expr {
    pub fn as_pipeline_item(self) -> Option<PipelineItem> {
        match self {
            Expr::Assign(assign) => Some(PipelineItem::Assigns(assign)),
            Expr::Cmd(cmd) => Some(PipelineItem::Cmd(cmd)),
            Expr::SubShell(sub_shell) => Some(PipelineItem::SubShell(sub_shell)),
            Expr::If(if_) => Some(PipelineItem::If(if_)),
            Expr::CondExpr(cond_expr) => Some(PipelineItem::CondExpr(cond_expr)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub label: ArcStr,
    pub value: Atom,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: Op,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub items: Vec<PipelineItem>,
}

#[derive(Debug, Clone)]
pub struct Cmd {
    pub assigns: Vec<Assign>,
    pub name_and_args: Vec<Atom>,
    pub redirect: Option<Redirect>,
    pub redirect_flags: RedirectFlags,
}

#[derive(Debug, Clone)]
pub struct SubShell {
    pub script: Script,
    pub redirect: Option<Redirect>,
    pub redirect_flags: RedirectFlags,
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Vec<Stmt>,
    pub then: Vec<Stmt>,
    /// From the spec:
    ///
    /// else_part        : Elif compound_list Then else_part
    ///                  | Else compound_list
    ///
    /// If len is:
    /// - 0                                   => no else
    /// - 1                                   => just else
    /// - 2n (n is # of elif/then branches)   => n elif/then branches
    /// - 2n + 1                              => n elif/then branches and an else branch
    pub else_parts: Vec<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct CondExpr {}

#[derive(Debug, Clone)]
pub enum Op {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum PipelineItem {
    Cmd(Cmd),
    Assigns(Vec<Assign>),
    SubShell(SubShell),
    If(If),
    CondExpr(CondExpr),
}

#[derive(Debug, Clone)]
pub enum CmdOrAssigns {
    Cmd(Cmd),
    Assigns(Vec<Assign>),
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct RedirectFlags {
    pub stdin: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub append: bool,
    pub duplicate_out: bool,
}

impl RedirectFlags {
    pub fn left() -> Self {
        Self {
            stdin: true,
            ..Default::default()
        }
    }

    pub fn leftleft() -> Self {
        Self {
            stdin: true,
            append: true,
            ..Default::default()
        }
    }

    pub fn right() -> Self {
        Self {
            stdout: true,
            ..Default::default()
        }
    }

    pub fn rightright() -> Self {
        Self {
            stdout: true,
            append: true,
            ..Default::default()
        }
    }

    pub fn andright() -> Self {
        Self {
            stdout: true,
            stderr: true,
            ..Default::default()
        }
    }

    pub fn andrightright() -> Self {
        Self {
            stdout: true,
            stderr: true,
            append: true,
            ..Default::default()
        }
    }

    pub fn two_right_and_one() -> Self {
        Self {
            stderr: true,
            duplicate_out: true,
            ..Default::default()
        }
    }

    pub fn one_right_and_two() -> Self {
        Self {
            stdout: true,
            duplicate_out: true,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        !(self.append || self.duplicate_out || self.stderr || self.stdin || self.stdout)
    }
}

#[derive(Debug, Clone)]
pub enum Redirect {
    Atom(Atom),
    PyObject,
}

#[derive(Debug, Clone)]
pub enum Atom {
    Simple(SimpleAtom),
    CompoundAtom(CompoundAtom),
}

#[derive(Debug, Clone)]
pub enum SimpleAtom {
    Var(ArcStr),
    VarArgv(u8),
    Text(ArcStr),
    Asterisk,
    DoubleAsterisk,
    BraceBegin,
    BraceEnd,
    Comma,
    Tilde,
    CmdSubst { script: Script, quoted: bool },
}

#[derive(Debug, Clone)]
pub struct CompoundAtom {
    pub atoms: Vec<SimpleAtom>,
    pub brace_expansion_hint: bool,
    pub glob_hint: bool,
}

impl Atom {
    pub fn simple(&self) -> Option<&SimpleAtom> {
        if let Atom::Simple(atom) = self {
            Some(atom)
        } else {
            None
        }
    }

    pub fn compound(&self) -> Option<&CompoundAtom> {
        if let Atom::CompoundAtom(atom) = self {
            Some(atom)
        } else {
            None
        }
    }

    pub fn merge(this: Atom, right: Atom) -> CompoundAtom {
        if this.simple().is_some() && right.simple().is_some() {
            let this = this.simple().unwrap().clone();
            let right = right.simple().unwrap().clone();
            let brace_expansion_hint =
                matches!(this, SimpleAtom::BraceBegin | SimpleAtom::BraceEnd)
                    || matches!(right, SimpleAtom::BraceBegin | SimpleAtom::BraceEnd);
            let glob_hint = matches!(this, SimpleAtom::Asterisk | SimpleAtom::DoubleAsterisk)
                || matches!(right, SimpleAtom::Asterisk | SimpleAtom::DoubleAsterisk);
            let atoms = vec![this, right];
            return CompoundAtom {
                atoms,
                brace_expansion_hint,
                glob_hint,
            };
        }

        if this.compound().is_some() && right.compound().is_some() {
            let this = this.compound().unwrap();
            let right = right.compound().unwrap();
            let atoms: Vec<_> = this
                .atoms
                .iter()
                .chain(right.atoms.iter())
                .cloned()
                .collect();
            return CompoundAtom {
                atoms,
                brace_expansion_hint: this.brace_expansion_hint || right.brace_expansion_hint,
                glob_hint: this.glob_hint || right.glob_hint,
            };
        }

        if this.simple().is_some() {
            let this = this.simple().unwrap();
            let right = right.compound().unwrap();
            let mut atoms = right.atoms.clone();
            atoms.insert(0, this.clone());
            return CompoundAtom {
                atoms,
                brace_expansion_hint: matches!(this, SimpleAtom::BraceBegin | SimpleAtom::BraceEnd)
                    || right.brace_expansion_hint,
                glob_hint: matches!(this, SimpleAtom::Asterisk | SimpleAtom::DoubleAsterisk)
                    || right.glob_hint,
            };
        }

        let this = this.compound().unwrap();
        let right = right.simple().unwrap();
        let mut atoms = this.atoms.clone();
        atoms.push(right.clone());
        return CompoundAtom {
            atoms,
            brace_expansion_hint: this.brace_expansion_hint
                || matches!(right, SimpleAtom::BraceBegin | SimpleAtom::BraceEnd),
            glob_hint: this.glob_hint
                || matches!(right, SimpleAtom::Asterisk | SimpleAtom::DoubleAsterisk),
        };
    }
}
