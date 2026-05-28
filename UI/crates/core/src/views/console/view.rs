use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConsoleSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ConsoleEntry {
    pub severity: ConsoleSeverity,
    pub message: String,
}

impl ConsoleEntry {
    pub fn info(msg: impl Into<String>) -> Self {
        Self {
            severity: ConsoleSeverity::Info,
            message: msg.into(),
        }
    }
    pub fn warning(msg: impl Into<String>) -> Self {
        Self {
            severity: ConsoleSeverity::Warning,
            message: msg.into(),
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            severity: ConsoleSeverity::Error,
            message: msg.into(),
        }
    }
}

pub struct ConsoleView {
    pub entries: Vec<ConsoleEntry>,
    pub show_info: bool,
    pub show_warning: bool,
    pub show_error: bool,
    pub max_entries: usize,
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            show_info: true,
            show_warning: true,
            show_error: true,
            max_entries: 10000,
        }
    }
}

impl ConsoleView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: ConsoleEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Console").with_icon(Icon::Info);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let row_h = metrics.row_height;

        let mut y = body.y;
        for entry in self.entries.iter().rev() {
            let visible = match entry.severity {
                ConsoleSeverity::Info => self.show_info,
                ConsoleSeverity::Warning => self.show_warning,
                ConsoleSeverity::Error => self.show_error,
            };
            if !visible {
                continue;
            }
            if y + row_h > body.y + body.height {
                break;
            }
            let color = match entry.severity {
                ConsoleSeverity::Info => palette.text_muted,
                ConsoleSeverity::Warning => palette.warning,
                ConsoleSeverity::Error => palette.error,
            };
            let prefix = match entry.severity {
                ConsoleSeverity::Info => Icon::Info.glyph(),
                ConsoleSeverity::Warning => Icon::Warning.glyph(),
                ConsoleSeverity::Error => Icon::Error.glyph(),
            };
            ui.draw_list.text(
                Vec2::new(body.x + metrics.padding, y + (row_h - metrics.font_size_normal) * 0.5),
                format!("{}  {}", prefix, entry.message),
                color,
                metrics.font_size_normal,
            );
            y += row_h;
        }
    }
}
