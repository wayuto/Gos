use crate::token::{Literal, TokenType};

#[derive(Debug)]
pub struct Program {
    pub body: Vec<Expr>,
}

#[derive(Debug)]
pub struct Stmt {
    pub body: Vec<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Stmt(Stmt),
    Val(Val),
    Var(Var),
    VarDecl(VarDecl),
    VarMod(VarMod),
    BinOp(BinOp),
    UnaryOp(UnaryOp),
    If(If),
    While(While),
    FuncDecl(FuncDecl),
    FuncCall(FuncCall),
    Return(Return),
    Out(Out),
    In(In),
    Label(Label),
    Goto(Goto),
    Exit(Exit),
}

#[derive(Debug)]
pub struct Val {
    pub value: Literal,
}

#[derive(Debug)]
pub struct Var {
    pub name: String,
}

#[derive(Debug)]
pub struct VarDecl {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct VarMod {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct BinOp {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub operator: TokenType,
}

#[derive(Debug)]
pub struct UnaryOp {
    pub argument: Box<Expr>,
    pub operator: TokenType,
}

#[derive(Debug)]
pub struct If {
    pub condition: Box<Expr>,
    pub then: Box<Expr>,
    pub else_branch: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct While {
    pub condition: Box<Expr>,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub struct FuncCall {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct Return {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct Out {
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct In {
    pub name: String,
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
}

#[derive(Debug)]
pub struct Goto {
    pub label: String,
}

#[derive(Debug)]
pub struct Exit {
    pub code: Option<Box<Expr>>,
}
