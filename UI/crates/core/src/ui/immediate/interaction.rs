#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Interaction {
    pub hovered: bool,
    pub pressed: bool,
    pub released: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub dragging: bool,
    pub focused: bool,
}

impl Interaction {
    pub const NONE: Self = Self {
        hovered: false,
        pressed: false,
        released: false,
        clicked: false,
        double_clicked: false,
        dragging: false,
        focused: false,
    };
}
