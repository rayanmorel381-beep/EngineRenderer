#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Push,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Call,
    Ret,
    Jmp,
    JmpIf,
    Load,
    Store,
    Halt,
    PushFloat,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Neg,
    Not,
    CallNative,
}

impl Opcode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0  => Some(Opcode::Push),
            1  => Some(Opcode::Pop),
            2  => Some(Opcode::Add),
            3  => Some(Opcode::Sub),
            4  => Some(Opcode::Mul),
            5  => Some(Opcode::Div),
            6  => Some(Opcode::Call),
            7  => Some(Opcode::Ret),
            8  => Some(Opcode::Jmp),
            9  => Some(Opcode::JmpIf),
            10 => Some(Opcode::Load),
            11 => Some(Opcode::Store),
            12 => Some(Opcode::Halt),
            13 => Some(Opcode::PushFloat),
            14 => Some(Opcode::Eq),
            15 => Some(Opcode::Ne),
            16 => Some(Opcode::Lt),
            17 => Some(Opcode::Gt),
            18 => Some(Opcode::Le),
            19 => Some(Opcode::Ge),
            20 => Some(Opcode::Neg),
            21 => Some(Opcode::Not),
            22 => Some(Opcode::CallNative),
            _  => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub op: Opcode,
    pub arg: i64,
}

impl Instruction {
    pub fn new(op: Opcode, arg: i64) -> Self {
        Self { op, arg }
    }
}
