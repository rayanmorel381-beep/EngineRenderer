use rhai::{Engine, Scope, AST};

pub struct ScriptRuntime {
    engine: Engine,
    compiled: Vec<Option<AST>>,
    last_sources: Vec<String>,
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(100_000);
        engine.set_max_call_levels(64);
        engine.set_max_string_size(65536);
        engine.set_max_array_size(4096);
        engine.set_max_map_size(1024);
        Self { engine, compiled: Vec::new(), last_sources: Vec::new() }
    }
}

impl ScriptRuntime {
    pub fn new() -> Self { Self::default() }

    pub fn ensure_slots(&mut self, count: usize) {
        while self.compiled.len() < count {
            self.compiled.push(None);
            self.last_sources.push(String::new());
        }
    }

    pub fn hot_reload(&mut self, index: usize, source: &str) {
        self.ensure_slots(index + 1);
        if self.last_sources[index] == source { return; }
        self.last_sources[index] = source.to_string();
        self.compiled[index] = match self.engine.compile(source) {
            Ok(ast) => Some(ast),
            Err(_) => None,
        };
    }

    pub fn call_on_update(&self, index: usize, dt: f64, position: &mut [f64; 3]) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        scope.push("dt", dt);
        scope.push("pos_x", position[0]);
        scope.push("pos_y", position[1]);
        scope.push("pos_z", position[2]);
        let result = self.engine.call_fn::<()>(&mut scope, ast, "on_update", (dt,));
        if result.is_ok() {
            if let Some(v) = scope.get_value::<f64>("pos_x") { position[0] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_y") { position[1] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_z") { position[2] = v; }
        }
    }

    pub fn is_compiled(&self, index: usize) -> bool {
        self.compiled.get(index).and_then(|s| s.as_ref()).is_some()
    }

    pub fn last_error_for(&self, index: usize, source: &str) -> Option<String> {
        self.ensure_slots_immut(index + 1)?;
        if self.last_sources.get(index)? != source { return None; }
        if self.compiled.get(index)?.is_none() {
            return Some("Erreur de compilation".to_string());
        }
        None
    }

    fn ensure_slots_immut(&self, count: usize) -> Option<()> {
        if self.compiled.len() >= count { Some(()) } else { None }
    }

    pub fn call_on_begin_play(&self, index: usize, position: &mut [f64; 3]) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        scope.push("pos_x", position[0]);
        scope.push("pos_y", position[1]);
        scope.push("pos_z", position[2]);
        let result = self.engine.call_fn::<()>(&mut scope, ast, "on_begin_play", ());
        if result.is_ok() {
            if let Some(v) = scope.get_value::<f64>("pos_x") { position[0] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_y") { position[1] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_z") { position[2] = v; }
        }
    }

    pub fn call_on_end_play(&self, index: usize) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        let _ = self.engine.call_fn::<()>(&mut scope, ast, "on_end_play", ());
    }

    pub fn call_on_hit(&self, index: usize, other_id: u64, normal: [f64; 3], position: &mut [f64; 3]) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        scope.push("other_id", other_id as i64);
        scope.push("normal_x", normal[0]);
        scope.push("normal_y", normal[1]);
        scope.push("normal_z", normal[2]);
        scope.push("pos_x", position[0]);
        scope.push("pos_y", position[1]);
        scope.push("pos_z", position[2]);
        let result = self.engine.call_fn::<()>(&mut scope, ast, "on_hit", (other_id as i64, normal[0], normal[1], normal[2]));
        if result.is_ok() {
            if let Some(v) = scope.get_value::<f64>("pos_x") { position[0] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_y") { position[1] = v; }
            if let Some(v) = scope.get_value::<f64>("pos_z") { position[2] = v; }
        }
    }

    pub fn call_on_overlap_begin(&self, index: usize, other_id: u64) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        let _ = self.engine.call_fn::<()>(&mut scope, ast, "on_overlap_begin", (other_id as i64,));
    }

    pub fn call_on_overlap_end(&self, index: usize, other_id: u64) {
        let Some(Some(ast)) = self.compiled.get(index) else { return };
        let mut scope = Scope::new();
        let _ = self.engine.call_fn::<()>(&mut scope, ast, "on_overlap_end", (other_id as i64,));
    }
}
