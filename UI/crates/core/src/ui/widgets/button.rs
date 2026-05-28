use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::immediate::interaction::Interaction;
use crate::ui::layout::rect::Rect;
use crate::ui::style::icons::Icon;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Ghost,
    Danger,
    IconOnly,
}

pub struct Button<'a> {
    pub label: &'a str,
    pub icon: Icon,
    pub style: ButtonStyle,
    pub enabled: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: Icon::None,
            style: ButtonStyle::Secondary,
            enabled: true,
        }
    }

    pub fn primary(label: &'a str) -> Self {
        Self::new(label).with_style(ButtonStyle::Primary)
    }

    pub fn icon(icon: Icon) -> Self {
        Self {
            label: "",
            icon,
            style: ButtonStyle::IconOnly,
            enabled: true,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect) -> Interaction {
        let pointer = (ui.input.pointer.x, ui.input.pointer.y);
        let hovered = self.enabled && rect.contains(crate::ui::layout::rect::Vec2::new(pointer.0, pointer.1));
        let pressed = hovered && ui.input.pointer.left_down;
        let interaction = Interaction {
            hovered,
            pressed,
            released: false,
            clicked: pressed && ui.active != id,
            double_clicked: false,
            dragging: false,
            focused: ui.focused == id,
        };

        if hovered {
            ui.set_hovered(id);
        }
        if pressed {
            ui.set_active(id);
        }

        let palette = ui.theme.palette;
        let bg = match (self.style, hovered, pressed) {
            (_, _, true) => palette.panel_active,
            (ButtonStyle::Primary, true, _) => palette.accent_hover,
            (ButtonStyle::Primary, false, _) => palette.accent,
            (ButtonStyle::Danger, _, _) => palette.error,
            (_, true, _) => palette.panel_hover,
            _ => palette.panel,
        };
        let text_color = if !self.enabled {
            palette.text_disabled
        } else {
            palette.text
        };

        ui.draw_list.rect(rect, bg, ui.theme.metrics.corner_radius);
        ui.draw_list
            .rect_outline(rect, palette.border, ui.theme.metrics.border_width, ui.theme.metrics.corner_radius);

        let label_text = if self.icon == Icon::None {
            self.label.to_string()
        } else if self.label.is_empty() {
            self.icon.glyph().to_string()
        } else {
            format!("{} {}", self.icon.glyph(), self.label)
        };
        ui.draw_list.text(
            crate::ui::layout::rect::Vec2::new(
                rect.x + ui.theme.metrics.padding,
                rect.y + (rect.height - ui.theme.metrics.font_size_normal) * 0.5,
            ),
            label_text,
            text_color,
            ui.theme.metrics.font_size_normal,
        );

        interaction
    }
}
