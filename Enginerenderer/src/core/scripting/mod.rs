pub mod bytecode;
pub mod compiler;
pub mod vm;

use std::collections::HashMap;
use std::path::Path;
use bytecode::Instruction;

#[derive(Debug)]
pub struct ScriptRunner {
    scripts: HashMap<String, Vec<Instruction>>,
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self { scripts: HashMap::new() }
    }

    pub fn load(&mut self, name: &str, src: &str) -> Result<(), String> {
        let instrs = compiler::compile_expr(src.trim())?;
        self.scripts.insert(name.to_string(), instrs);
        Ok(())
    }

    pub fn load_from_file(&mut self, name: &str, path: &Path) -> Result<(), String> {
        let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.load(name, &src)
    }

    pub fn run(&self, name: &str, vm: &mut vm::Vm) -> Option<Result<vm::Value, vm::VmError>> {
        self.scripts.get(name).map(|prog| vm.run(prog))
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }

    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }
}
