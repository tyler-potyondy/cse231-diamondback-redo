pub const TRUE_VAL:u64  = 3;
pub const FALSE_VAL:u64 = 1;
pub const OVERFLOW_ERROR_CODE:u64 = 5;
pub const INVALID_ARGUMENT_ERROR_CODE:u64 = 7;
pub const GREATEST_VAL:i64 = 4611686018427387903;
pub const LEAST_VAL:i64 = -4611686018427387904;


#[derive(Debug)]
pub enum Val {
    Reg(Reg),
    Imm(u64),
    RegOffset(Reg, i64),
    Label(String),
}

#[derive(Debug)]
pub enum Reg {
    RAX,
    RSP,
    RDI,
    RBX,
}

pub struct Program {
    pub defs: Vec<Definition>,
    pub main: Expr,
}

#[derive(Debug)]
pub enum Definition {
    Fun1(String, String, Expr),
    Fun2(String, String, String, Expr),
}

#[derive(Debug)]
pub enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
    Shr(Val,Val),
    Shl(Val,Val),
    Jmp(Val),
    Cmp(Val,Val),
    JEqual(Val),
    JNotEqual(Val),
    JGreater(Val),
    JGreaterEqual(Val),
    JLess(Val),
    JLessEqual(Val),
    Test(Val,Val),
    Label(Val),
    Xor(Val,Val),
    Cmove(Val,Val),
    OverFlow(),
    Call(Val),
    Push(Val),
    Pop(Val),

}

#[derive(Debug)]
pub enum Op1 {
    Add1,
    Sub1,
    IsNum,
    IsBool,
    Print,
}

#[derive(Debug)]
pub enum Op2 {
    Plus,
    Minus,
    Times,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug)]
pub enum Expr {
    Number(u64),
    Boolean(bool),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Set(String, Box<Expr>),
    Block(Vec<Expr>),

    Call1(String, Box<Expr>),
    Call2(String, Box<Expr>, Box<Expr>),}