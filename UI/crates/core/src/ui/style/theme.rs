use crate::ui::style::metrics::Metrics;
use crate::ui::style::palette::Palette;

pub struct Theme {
    pub palette: Palette,
    pub metrics: Metrics,
}

impl Theme {
    pub const DARK: Self = Self {
        palette: Palette::DARK,
        metrics: Metrics::DEFAULT,
    };

    pub const LIGHT: Self = Self {
        palette: Palette::LIGHT,
        metrics: Metrics::DEFAULT,
    };

    pub const DARK_COMPACT: Self = Self {
        palette: Palette::DARK,
        metrics: Metrics::COMPACT,
    };
}
