use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::Rect;
use crate::ui::panels::title_bar::TitleBar;
use crate::ui::style::icons::Icon;

#[derive(Copy, Clone, Debug, Default)]
pub struct PanelFlags {
    pub no_title_bar: bool,
    pub no_background: bool,
    pub no_border: bool,
    pub no_padding: bool,
}

pub struct Panel<'a> {
    pub title: &'a str,
    pub icon: Icon,
    pub flags: PanelFlags,
}

impl<'a> Panel<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: Icon::None,
            flags: PanelFlags::default(),
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn flags(mut self, flags: PanelFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn body_rect(&self, ui: &UiContext, rect: Rect) -> Rect {
        let metrics = ui.theme.metrics;
        let mut body = rect;
        if !self.flags.no_title_bar {
            let (_title, rest) = body.split_top(metrics.title_bar_height);
            body = rest;
        }
        if !self.flags.no_padding {
            body = body.shrink(metrics.padding_small);
        }
        body
    }

    pub fn show_chrome(&self, ui: &mut UiContext, rect: Rect) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        if !self.flags.no_background {
            let shadow_color = [0.0, 0.0, 0.0, 0.45];
            let shadow_rect = Rect::new(rect.x, rect.y + 2.0, rect.width, rect.height);
            ui.draw_list.shadow(
                shadow_rect,
                shadow_color,
                metrics.corner_radius,
                metrics.shadow_spread,
            );
            ui.draw_list.rect(rect, palette.panel, metrics.corner_radius);
        }
        if !self.flags.no_border {
            ui.draw_list.rect_outline(rect, palette.border, metrics.border_width, metrics.corner_radius);
        }
        if self.flags.no_title_bar {
            return false;
        }
        let (title_rect, _) = rect.split_top(metrics.title_bar_height);
        TitleBar::new(self.title).with_icon(self.icon).show(ui, title_rect)
    }
}
