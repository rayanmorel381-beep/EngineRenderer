#[derive(Clone, Debug, PartialEq)]
pub enum InputDevice {
    Keyboard,
    Mouse,
    Gamepad,
    Touch,
}

impl InputDevice {
    pub fn label(&self) -> &'static str {
        match self { Self::Keyboard => "Clavier", Self::Mouse => "Souris", Self::Gamepad => "Manette", Self::Touch => "Tactile" }
    }
    pub const ALL: [InputDevice; 4] = [InputDevice::Keyboard, InputDevice::Mouse, InputDevice::Gamepad, InputDevice::Touch];
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputTrigger {
    Pressed,
    Released,
    Held,
    Axis,
}

impl InputTrigger {
    pub fn label(&self) -> &'static str {
        match self { Self::Pressed => "Appuyé", Self::Released => "Relâché", Self::Held => "Maintenu", Self::Axis => "Axe" }
    }
    pub const ALL: [InputTrigger; 4] = [InputTrigger::Pressed, InputTrigger::Released, InputTrigger::Held, InputTrigger::Axis];
}

#[derive(Clone, Debug)]
pub struct InputBinding {
    pub key: String,
    pub device: InputDevice,
    pub trigger: InputTrigger,
    pub scale: f64,
    pub alt_key: Option<String>,
    pub modifier_ctrl: bool,
    pub modifier_shift: bool,
    pub modifier_alt: bool,
}

impl InputBinding {
    pub fn new(key: impl Into<String>, device: InputDevice, trigger: InputTrigger) -> Self {
        Self {
            key: key.into(), device, trigger, scale: 1.0,
            alt_key: None, modifier_ctrl: false, modifier_shift: false, modifier_alt: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputAction {
    pub name: String,
    pub description: String,
    pub bindings: Vec<InputBinding>,
    pub consume_input: bool,
    pub enabled: bool,
}

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), description: String::new(), bindings: Vec::new(), consume_input: true, enabled: true }
    }
}

#[derive(Clone, Debug)]
pub struct InputAxis {
    pub name: String,
    pub bindings: Vec<InputBinding>,
    pub dead_zone: f64,
    pub sensitivity: f64,
    pub invert: bool,
}

impl InputAxis {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), bindings: Vec::new(), dead_zone: 0.1, sensitivity: 1.0, invert: false }
    }
}

#[derive(Clone, Debug)]
pub struct InputContext {
    pub name: String,
    pub priority: i32,
    pub active: bool,
}

impl InputContext {
    pub fn new(name: impl Into<String>, priority: i32) -> Self {
        Self { name: name.into(), priority, active: true }
    }
}

#[derive(Clone, Debug)]
pub struct InputMap {
    pub actions: Vec<InputAction>,
    pub axes: Vec<InputAxis>,
    pub contexts: Vec<InputContext>,
    pub active_context: usize,
}

impl Default for InputMap {
    fn default() -> Self {
        let mut map = Self { actions: Vec::new(), axes: Vec::new(), contexts: Vec::new(), active_context: 0 };
        map.contexts.push(InputContext::new("Default", 0));
        map.contexts.push(InputContext::new("UI", 100));

        let mut jump = InputAction::new("Sauter");
        jump.description = "Faire sauter le personnage".to_string();
        jump.bindings.push(InputBinding::new("Space", InputDevice::Keyboard, InputTrigger::Pressed));
        jump.bindings.push(InputBinding::new("A", InputDevice::Gamepad, InputTrigger::Pressed));
        map.actions.push(jump);

        let mut attack = InputAction::new("Attaquer");
        attack.bindings.push(InputBinding::new("MouseLeft", InputDevice::Mouse, InputTrigger::Pressed));
        attack.bindings.push(InputBinding::new("X", InputDevice::Gamepad, InputTrigger::Pressed));
        map.actions.push(attack);

        let mut interact = InputAction::new("Interagir");
        interact.bindings.push(InputBinding::new("E", InputDevice::Keyboard, InputTrigger::Pressed));
        interact.bindings.push(InputBinding::new("B", InputDevice::Gamepad, InputTrigger::Pressed));
        map.actions.push(interact);

        let mut move_h = InputAxis::new("Déplacement H");
        move_h.bindings.push(InputBinding::new("D", InputDevice::Keyboard, InputTrigger::Axis));
        move_h.bindings.push({ let mut b = InputBinding::new("A", InputDevice::Keyboard, InputTrigger::Axis); b.scale = -1.0; b });
        move_h.bindings.push(InputBinding::new("LeftStickX", InputDevice::Gamepad, InputTrigger::Axis));
        map.axes.push(move_h);

        let mut move_v = InputAxis::new("Déplacement V");
        move_v.bindings.push(InputBinding::new("W", InputDevice::Keyboard, InputTrigger::Axis));
        move_v.bindings.push({ let mut b = InputBinding::new("S", InputDevice::Keyboard, InputTrigger::Axis); b.scale = -1.0; b });
        move_v.bindings.push(InputBinding::new("LeftStickY", InputDevice::Gamepad, InputTrigger::Axis));
        map.axes.push(move_v);

        map
    }
}

impl InputMap {
    pub fn new() -> Self { Self::default() }

    pub fn add_action(&mut self, action: InputAction) {
        self.actions.push(action);
    }

    pub fn add_axis(&mut self, axis: InputAxis) {
        self.axes.push(axis);
    }

    pub fn remove_action(&mut self, index: usize) {
        if index < self.actions.len() { self.actions.remove(index); }
    }

    pub fn remove_axis(&mut self, index: usize) {
        if index < self.axes.len() { self.axes.remove(index); }
    }

    pub fn find_action(&self, name: &str) -> Option<&InputAction> {
        self.actions.iter().find(|a| a.name == name)
    }

    pub fn find_axis(&self, name: &str) -> Option<&InputAxis> {
        self.axes.iter().find(|a| a.name == name)
    }
}
