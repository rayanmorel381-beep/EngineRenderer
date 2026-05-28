pub type Rgba = [f64; 4];

#[derive(Copy, Clone, Debug)]
pub struct Palette {
    pub background: Rgba,
    pub panel: Rgba,
    pub panel_hover: Rgba,
    pub panel_active: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub text_disabled: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub error: Rgba,
    pub selection: Rgba,
    pub viewport_clear: Rgba,
}

impl Palette {
    pub const DARK: Self = Self {
        background: [0.043, 0.047, 0.063, 1.0],
        panel: [0.078, 0.086, 0.110, 0.96],
        panel_hover: [0.110, 0.118, 0.149, 0.98],
        panel_active: [0.149, 0.157, 0.196, 1.0],
        border: [0.196, 0.208, 0.255, 0.85],
        accent: [0.984, 0.580, 0.176, 1.0],
        accent_hover: [1.000, 0.671, 0.290, 1.0],
        text: [0.918, 0.929, 0.961, 1.0],
        text_muted: [0.561, 0.588, 0.659, 1.0],
        text_disabled: [0.345, 0.376, 0.443, 1.0],
        success: [0.345, 0.808, 0.529, 1.0],
        warning: [0.984, 0.749, 0.286, 1.0],
        error: [0.953, 0.353, 0.380, 1.0],
        selection: [0.984, 0.580, 0.176, 0.32],
        viewport_clear: [0.020, 0.024, 0.035, 1.0],
    };

    pub const LIGHT: Self = Self {
        background: [0.961, 0.965, 0.980, 1.0],
        panel: [0.918, 0.929, 0.953, 1.0],
        panel_hover: [0.871, 0.886, 0.918, 1.0],
        panel_active: [0.812, 0.831, 0.871, 1.0],
        border: [0.706, 0.722, 0.761, 1.0],
        accent: [0.945, 0.494, 0.106, 1.0],
        accent_hover: [1.000, 0.580, 0.176, 1.0],
        text: [0.094, 0.106, 0.137, 1.0],
        text_muted: [0.376, 0.408, 0.467, 1.0],
        text_disabled: [0.624, 0.643, 0.694, 1.0],
        success: [0.196, 0.659, 0.345, 1.0],
        warning: [0.851, 0.604, 0.094, 1.0],
        error: [0.831, 0.196, 0.220, 1.0],
        selection: [0.945, 0.494, 0.106, 0.28],
        viewport_clear: [0.851, 0.859, 0.886, 1.0],
    };
}
