use super::bytecode::{Instruction, Opcode};

pub type NativeFunc = Box<dyn Fn(&[Value]) -> Value + Send>;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn to_float(&self) -> f64 {
        match self {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Nil => 0.0,
        }
    }

    fn to_int(&self) -> i64 {
        match self {
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            Value::Bool(b) => if *b { 1 } else { 0 },
            Value::Nil => 0,
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Nil => false,
        }
    }

    fn eq_value(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Float(_), _) | (_, Value::Float(_)) => self.to_float() == other.to_float(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    fn lt_value(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a < b,
            _ => self.to_float() < other.to_float(),
        }
    }
}

#[derive(Debug)]
pub enum VmError {
    StackUnderflow,
    InvalidOpcode,
    DivisionByZero,
    InvalidJump,
    LocalOutOfBounds,
    UnknownNative(u64),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::StackUnderflow => f.write_str("stack underflow"),
            VmError::InvalidOpcode => f.write_str("invalid opcode"),
            VmError::DivisionByZero => f.write_str("division by zero"),
            VmError::InvalidJump => f.write_str("invalid jump target"),
            VmError::LocalOutOfBounds => f.write_str("local index out of bounds"),
            VmError::UnknownNative(id) => write!(f, "unknown native function id {}", id),
        }
    }
}

#[derive(Debug)]
struct VmStack {
    data: Vec<Value>,
}

impl VmStack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, v: Value) {
        self.data.push(v);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.data.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.data.last().ok_or(VmError::StackUnderflow)
    }
}

pub struct Vm {
    stack: VmStack,
    locals: Vec<Value>,
    ip: usize,
    call_stack: Vec<usize>,
    natives: Vec<(u64, NativeFunc)>,
}

impl Vm {
    pub fn new(local_count: usize) -> Self {
        Self {
            stack: VmStack::new(),
            locals: vec![Value::Nil; local_count],
            ip: 0,
            call_stack: Vec::new(),
            natives: Vec::new(),
        }
    }

    pub fn register_native<F: Fn(&[Value]) -> Value + Send + 'static>(&mut self, id: u64, func: F) {
        self.natives.push((id, Box::new(func)));
    }

    pub fn run(&mut self, program: &[Instruction]) -> Result<Value, VmError> {
        self.ip = 0;
        loop {
            if self.ip >= program.len() {
                break;
            }
            let instr = program[self.ip];
            self.ip += 1;
            match instr.op {
                Opcode::Push => {
                    self.stack.push(Value::Int(instr.arg));
                }
                Opcode::Pop => {
                    self.stack.pop()?;
                }
                Opcode::Add => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let result = match (&a, &b) {
                        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(a.to_float() + b.to_float()),
                        _ => Value::Int(a.to_int().wrapping_add(b.to_int())),
                    };
                    self.stack.push(result);
                }
                Opcode::Sub => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let result = match (&a, &b) {
                        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(a.to_float() - b.to_float()),
                        _ => Value::Int(a.to_int().wrapping_sub(b.to_int())),
                    };
                    self.stack.push(result);
                }
                Opcode::Mul => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    let result = match (&a, &b) {
                        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(a.to_float() * b.to_float()),
                        _ => Value::Int(a.to_int().wrapping_mul(b.to_int())),
                    };
                    self.stack.push(result);
                }
                Opcode::Div => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    match &b {
                        Value::Int(0) => return Err(VmError::DivisionByZero),
                        Value::Float(f) if *f == 0.0 => return Err(VmError::DivisionByZero),
                        _ => {}
                    }
                    let result = match (&a, &b) {
                        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(a.to_float() / b.to_float()),
                        _ => Value::Int(a.to_int().wrapping_div(b.to_int())),
                    };
                    self.stack.push(result);
                }
                Opcode::Call => {
                    self.call_stack.push(self.ip);
                    let target = instr.arg as usize;
                    if target >= program.len() {
                        return Err(VmError::InvalidJump);
                    }
                    self.ip = target;
                }
                Opcode::Ret => {
                    if let Some(return_addr) = self.call_stack.pop() {
                        self.ip = return_addr;
                    } else {
                        break;
                    }
                }
                Opcode::Jmp => {
                    let target = instr.arg as usize;
                    if target >= program.len() {
                        return Err(VmError::InvalidJump);
                    }
                    self.ip = target;
                }
                Opcode::JmpIf => {
                    let normalized = Value::Bool(self.stack.peek()?.is_truthy());
                    let truthy = normalized.is_truthy();
                    self.stack.pop()?;
                    if truthy {
                        let target = instr.arg as usize;
                        if target >= program.len() {
                            return Err(VmError::InvalidJump);
                        }
                        self.ip = target;
                    }
                }
                Opcode::Load => {
                    let idx = instr.arg as usize;
                    if idx >= self.locals.len() {
                        return Err(VmError::LocalOutOfBounds);
                    }
                    self.stack.push(self.locals[idx].clone());
                }
                Opcode::Store => {
                    let idx = instr.arg as usize;
                    if idx >= self.locals.len() {
                        return Err(VmError::LocalOutOfBounds);
                    }
                    let val = self.stack.pop()?;
                    self.locals[idx] = val;
                }
                Opcode::Halt => break,
                Opcode::PushFloat => {
                    let f = f64::from_bits(instr.arg as u64);
                    self.stack.push(Value::Float(f));
                }
                Opcode::Eq => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(a.eq_value(&b)));
                }
                Opcode::Ne => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(!a.eq_value(&b)));
                }
                Opcode::Lt => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(a.lt_value(&b)));
                }
                Opcode::Gt => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(b.lt_value(&a)));
                }
                Opcode::Le => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(!b.lt_value(&a)));
                }
                Opcode::Ge => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(!a.lt_value(&b)));
                }
                Opcode::Neg => {
                    let a = self.stack.pop()?;
                    let result = match a {
                        Value::Int(i) => Value::Int(i.wrapping_neg()),
                        Value::Float(f) => Value::Float(-f),
                        other => Value::Float(-other.to_float()),
                    };
                    self.stack.push(result);
                }
                Opcode::Not => {
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Bool(!a.is_truthy()));
                }
                Opcode::CallNative => {
                    let arg = self.stack.pop()?;
                    let result = self.call_native(instr.arg as u64, &[arg])?;
                    self.stack.push(result);
                }
            }
        }
        self.stack.data.pop().map(Ok).unwrap_or(Ok(Value::Nil))
    }

    pub fn decode_and_run(&mut self, bytes: &[u8]) -> Result<Value, VmError> {
        let mut instrs: Vec<super::bytecode::Instruction> = Vec::new();
        let mut i = 0;
        while i + 8 < bytes.len() {
            let op = super::bytecode::Opcode::from_u8(bytes[i]).ok_or(VmError::InvalidOpcode)?;
            let arg = i64::from_le_bytes(bytes[i + 1..i + 9].try_into().unwrap_or([0u8; 8]));
            instrs.push(super::bytecode::Instruction::new(op, arg));
            i += 9;
        }
        self.run(&instrs)
    }

    pub fn call_native(&self, id: u64, args: &[Value]) -> Result<Value, VmError> {
        for (nid, func) in &self.natives {
            if *nid == id {
                return Ok(func(args));
            }
        }
        Err(VmError::UnknownNative(id))
    }
}

impl std::fmt::Debug for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("locals_len", &self.locals.len())
            .field("stack_depth", &self.stack.data.len())
            .field("ip", &self.ip)
            .field("native_count", &self.natives.len())
            .finish()
    }
}
