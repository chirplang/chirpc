use std::fmt::{Debug, Error, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChipParseError<'a> {
    #[error("Tag was closed incorrectly: {0:?} != {1:?}")]
    TagClosedIncorrectly(Ident<'a>, Ident<'a>),
    #[error("Integer literal was too large (couldn't fit into i64): {0}")]
    IntTooLarge(&'a str),
    #[error("Float literal was too large (couldn't fit into f64): {0}")]
    FloatTooLarge(&'a str),
}

pub struct CompilationUnit<'a>(pub Vec<TopLevelDef<'a>>);

pub enum TopLevelDef<'a> {
    Func(FunctionDef<'a>),
    Export(Ident<'a>, TagList<'a>),
    TagDef(TagDef<'a>),
}

pub struct TagDef<'a> {
    pub ident: Ident<'a>,
    pub states: StateList<'a>,
    pub fns: FunctionDefList<'a>,
}

pub struct StateList<'a>(pub Vec<State<'a>>);

pub struct State<'a> {
    pub ident: Ident<'a>,
    pub chip_type: Option<Ident<'a>>,
    pub default: Option<Box<Statement<'a>>>,
}

pub struct FunctionDefList<'a>(pub Vec<FunctionDef<'a>>);

pub struct FunctionDef<'a> {
    pub ident: Ident<'a>,
    pub args: ArgDefList<'a>,
    pub return_type: Option<Ident<'a>>,
    pub block: StatementList<'a>,
}

pub struct ArgDefList<'a>(pub Vec<ArgDef<'a>>);

pub struct ArgDef<'a> {
    pub name: Ident<'a>,
    pub chip_type: Ident<'a>,
}

pub struct StatementList<'a>(pub Vec<Box<Statement<'a>>>);

pub enum Statement<'a> {
    Number(Number),
    Op(Box<Statement<'a>>, Opcode, Box<Statement<'a>>),
    FunctionCall(FunctionCall<'a>),
    If(Box<Statement<'a>>, StatementList<'a>),
    IfElse(Box<Statement<'a>>, StatementList<'a>, StatementList<'a>),
    Let(Ident<'a>),
    LetAssign(Ident<'a>, Box<Statement<'a>>),
    Assign(IdentList<'a>, Box<Statement<'a>>),
    Tag(Tag<'a>),
    Ident(Ident<'a>),
    Block(StatementList<'a>),
    Error,
}

pub struct FunctionCall<'a> {
    pub ident: Ident<'a>,
    pub args: ArgList<'a>,
}

pub struct ArgList<'a>(pub Vec<Box<Statement<'a>>>);

pub struct IdentList<'a>(pub Vec<Ident<'a>>);

pub struct TagList<'a>(pub Vec<Tag<'a>>);

pub struct Tag<'a> {
    pub ident: Ident<'a>,
    pub properties: PropertyList<'a>,
    pub children: TagList<'a>,
}

pub struct PropertyList<'a>(pub Vec<(Ident<'a>, Ident<'a>)>);

#[derive(PartialEq, Eq)]
pub struct Ident<'a>(pub &'a str);

pub struct Struct<'a> {
    pub ident: Ident<'a>,
    pub generics: Vec<Ident<'a>>,
}

pub struct Type<'a> {
    pub ident: Ident<'a>,
    pub generics: Vec<Ident<'a>>,
}

pub enum Number {
    Int(i64),
    Float(f64),
}

pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

impl Debug for CompilationUnit<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut builder = String::new();
        for item in &self.0 {
            builder.push_str(&format!("{:?}", item));
        }
        return write!(f, "{}", builder);
    }
}

impl Debug for TopLevelDef<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::TopLevelDef::*;
        match self {
            Func(fun) => write!(f, "{:?}", fun),
            Export(i, t) => write!(f, "export {:?} {{\n{:?}}}\n", i, t),
            TagDef(t) => write!(f, "tag {:?} {{\n{:?}\n{:?}}}", t.ident, t.states, t.fns),
        }
    }
}

impl Debug for TagDef<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "tag {:?} {{\n", self.ident)?;
        for state in &self.states.0 {
            write!(f, "{:?}\n", state)?;
        }
        for func in &self.fns.0 {
            write!(f, "{:?}\n", func)?;
        }
        write!(f, "}}\n")
    }
}

impl Debug for StateList<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for item in &self.0 {
            write!(f, "{:?}\n", item)?;
        }
        Ok(())
    }
}

impl Debug for State<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "state {:?}", self.ident)?;
        if let Some(t) = &self.chip_type {
            write!(f, ": {:?}", t)?;
        }
        if let Some(s) = &self.default {
            write!(f, " = {:?}", s)?;
        }
        Ok(())
    }
}

impl Debug for FunctionDefList<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for fun in &self.0 {
            write!(f, "{:?}\n", fun)?;
        }
        Ok(())
    }
}

impl Debug for FunctionDef<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if let Some(ret) = &self.return_type {
            write!(
                f,
                "{:?}({:?}) -> {:?} {:?}",
                self.ident, self.args, ret, self.block
            )
        } else {
            write!(f, "{:?}({:?}) {:?}", self.ident, self.args, self.block)
        }
    }
}

impl Debug for ArgDefList<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut builder = String::new();
        for item in &self.0[0..self.0.len() - 1] {
            builder.push_str(&format!("{:?}, ", item));
        }
        builder.push_str(&format!("{:?}", self.0[self.0.len() - 1]));
        write!(f, "{}", builder)
    }
}

impl Debug for ArgDef<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}: {:?}", self.name, self.chip_type)
    }
}

impl Debug for StatementList<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            return write!(fmt, "{{ \n }}");
        }
        let mut comma_separated = String::from('\n');

        for item in &self.0[0..self.0.len() - 1] {
            comma_separated.push_str(&format!("{:?};\n", item));
        }

        comma_separated.push_str(&format!("{:?}", self.0[self.0.len() - 1]));
        write!(fmt, "{{ {};\n }}", comma_separated)
    }
}

impl Debug for Statement<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match self {
            Number(n) => write!(fmt, "{:?}", n),
            Op(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            FunctionCall(f) => write!(fmt, "{:?}", f),
            If(cond, exprs) => write!(fmt, "if {:?} {:?}", cond, exprs),
            IfElse(cond, if_exprs, else_exprs) => {
                write!(fmt, "if {:?} {:?} else {:?}", cond, if_exprs, else_exprs)
            }
            Let(i) => write!(fmt, "let {:?}", i),
            LetAssign(i, a) => write!(fmt, "let {:?} = {:?}", i, a),
            Assign(l, r) => write!(fmt, "{:?} = {:?}", l, r),
            Tag(t) => write!(fmt, "{:?}", t),
            Ident(i) => write!(fmt, "{:?}", i),
            Error => write!(fmt, "error"),
            Block(list) => write!(fmt, "{:?}", list),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Number::*;
        match *self {
            Int(i) => write!(fmt, "{}", i),
            Float(i) => write!(fmt, "{}", i),
        }
    }
}

impl Debug for FunctionCall<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}{:?}", self.ident, self.args)
    }
}

impl Debug for ArgList<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            return write!(fmt, "()");
        }
        let mut comma_separated = String::new();

        for item in &self.0[0..self.0.len() - 1] {
            comma_separated.push_str(&format!("{:?}, ", item));
        }

        comma_separated.push_str(&format!("{:?}", self.0[self.0.len() - 1]));
        write!(fmt, "({})", comma_separated)
    }
}

impl Debug for IdentList<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            // This should never happen
            return Err(Error);
        }
        let mut dot_separated = String::new();

        for item in &self.0[0..self.0.len() - 1] {
            dot_separated.push_str(&format!("{:?}.", item));
        }

        dot_separated.push_str(&format!("{:?}", self.0[self.0.len() - 1]));
        write!(fmt, "{}", dot_separated)
    }
}

impl Debug for TagList<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            return Err(Error);
        }
        let mut builder = String::from(" ");

        for item in &self.0[0..self.0.len() - 1] {
            builder.push_str(&format!("{:?} ", item));
        }

        let last = &self.0[self.0.len() - 1];
        builder.push_str(&format!("{:?} ", last));
        write!(fmt, "{}", builder)
    }
}

impl Debug for Tag<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.children.0.is_empty() {
            write!(fmt, "<{:?}{:?}/>", self.ident, self.properties)
        } else {
            write!(
                fmt,
                "<{:?}{:?}>{:?}</{:?}>",
                self.ident, self.properties, self.children, self.ident
            )
        }
    }
}

impl Debug for PropertyList<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }
        let mut builder = String::from(" ");

        for item in &self.0[0..self.0.len() - 1] {
            builder.push_str(&format!("{:?}={:?} ", item.0, item.1));
        }

        let last = &self.0[self.0.len() - 1];
        builder.push_str(&format!("{:?}={:?}", last.0, last.1));
        write!(fmt, "{}", builder)
    }
}

impl Debug for Ident<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", self.0)
    }
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match *self {
            Mul => write!(fmt, "*"),
            Div => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Sub => write!(fmt, "-"),
            Gt => write!(fmt, ">"),
            Ge => write!(fmt, ">="),
            Lt => write!(fmt, "<"),
            Le => write!(fmt, "<="),
            Eq => write!(fmt, "=="),
            Ne => write!(fmt, "!="),
        }
    }
}
