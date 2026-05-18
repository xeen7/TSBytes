use std::collections::HashMap;
use crate::codegen::compiler::ir::{Type, BinOp, UnaryOp};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TempId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Int(i64),
    Double(f64),
    String(String),
    Bool(bool),
    Null,
    Undefined,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Temp(TempId, Type),
    Local(String, Type),
    Const(Constant, Type),
    This(Type),
}

impl Operand {
    pub fn get_type(&self) -> Type {
        match self {
            Operand::Temp(_, t) => t.clone(),
            Operand::Local(_, t) => t.clone(),
            Operand::Const(_, t) => t.clone(),
            Operand::This(t) => t.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MirExpr {
    Operand(Operand),
    BinOp(BinOp, Operand, Operand, Type),
    UnaryOp(UnaryOp, Operand, Type),
    FieldGet(Operand, String, Type),
    IndexGet(Operand, Operand, Type),
    ArrayLit(Vec<Operand>, Type),
    ObjectLit(Vec<(String, Operand)>, Type),
    Cast(Operand, Type),
    InstanceOf(Operand, String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MirInstr {
    Assign(Operand, MirExpr),
    FieldSet(Operand, String, Operand),
    IndexSet(Operand, Operand, Operand),
    Call(Option<Operand>, String, Vec<Operand>, Type),
    MethodCall(Option<Operand>, Operand, String, Vec<Operand>, Type),
    SuperCall(String, Vec<Operand>, Type),
    New(Option<Operand>, String, Vec<Operand>, Type),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Terminator {
    Goto(BasicBlockId),
    Branch(Operand, BasicBlockId, BasicBlockId),
    Return(Option<Operand>),
    Throw(Operand),
    /// Unreachable block end (e.g. after throw or infinite loop)
    Unreachable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instrs: Vec<MirInstr>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug)]
pub struct MirFunction {
    pub name: String,
    pub blocks: HashMap<BasicBlockId, BasicBlock>,
    pub entry_block: BasicBlockId,
}

impl MirFunction {
    pub fn new(name: String) -> Self {
        Self {
            name,
            blocks: HashMap::new(),
            entry_block: BasicBlockId(0),
        }
    }
}
