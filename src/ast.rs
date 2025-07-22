use arcstr::ArcStr;

pub struct Script {
    pub stmts: Vec<Stmt>,
}

pub struct Stmt {
    pub exprs: Vec<Expr>,
}

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

pub struct Assign {
    pub label: ArcStr,
    pub value: Atom,
}

pub struct Binary {
    pub op: Op,
    pub left: Expr,
    pub right: Expr,
}

pub struct Pipeline {
    pub items: Vec<PipelineItem>,
}

pub struct Cmd {
    pub assigns: Vec<Assign>,
    pub name_and_args: Vec<Atom>,
    pub redirect: Option<Redirect>,
    pub redirect_flags: RedirectFlags,
}

pub struct SubShell {
    pub script: Script,
    pub redirect: Option<Redirect>,
    pub redirect_flags: RedirectFlags,
}

pub struct If {
    cond: Vec<Stmt>,
    then: Vec<Stmt>,
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
    else_parts: Vec<Vec<Stmt>>,
}

pub struct CondExpr {}

pub enum Op {
    And,
    Or,
}

pub enum PipelineItem {
    Cmd(Cmd),
    Assigns(Vec<Assign>),
    SubShell(SubShell),
    If(If),
    CondExpr(CondExpr),
}

pub enum CmdOrAssigns {
    Cmd(Cmd),
    Assigns(Vec<Assign>),
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
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

pub enum Redirect {
    Atom(Atom),
    PyObject,
}

pub enum Atom {
    Simple(SimpleAtom),
    CompoundAtom(CompoundAtom),
}

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

pub struct CompoundAtom {
    pub atoms: Vec<SimpleAtom>,
    pub brace_expansion_hint: bool,
    pub glob_hint: bool,
}
