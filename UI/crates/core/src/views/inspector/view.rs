use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::Rect;
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{PropertyGrid, PropertyRow, Slider, VectorInput};

pub struct InspectorView {
    pub object_name: String,
    pub object_type: String,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub material_intensity: f64,
    pub extra_rows: Vec<PropertyRow>,
}

impl Default for InspectorView {
    fn default() -> Self {
        Self {
            object_name: "<no selection>".to_string(),
            object_type: "None".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            material_intensity: 1.0,
            extra_rows: Vec::new(),
        }
    }
}

impl InspectorView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Inspector").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let row_h = ui.theme.metrics.row_height;

        let header_rows = vec![
            PropertyRow::new("Name", self.object_name.clone()),
            PropertyRow::new("Type", self.object_type.clone()),
        ];
        let header_rect = Rect::new(body.x, body.y, body.width, row_h * header_rows.len() as f64);
        PropertyGrid::new(&header_rows).show(ui, WidgetId::hash_str("inspector_header"), header_rect);

        let mut y = header_rect.y + header_rect.height + ui.theme.metrics.spacing;
        let id = WidgetId::hash_str("inspector_transform");
        let pos_rect = Rect::new(body.x, y, body.width, row_h);
        VectorInput::new("Position", 3).range(-1000.0, 1000.0).show(
            ui,
            id.child("pos"),
            pos_rect,
            &mut self.position,
        );
        y += row_h + ui.theme.metrics.spacing;

        let rot_rect = Rect::new(body.x, y, body.width, row_h);
        VectorInput::new("Rotation", 3)
            .range(-360.0, 360.0)
            .show(ui, id.child("rot"), rot_rect, &mut self.rotation);
        y += row_h + ui.theme.metrics.spacing;

        let scale_rect = Rect::new(body.x, y, body.width, row_h);
        VectorInput::new("Scale", 3)
            .range(0.01, 100.0)
            .show(ui, id.child("scale"), scale_rect, &mut self.scale);
        y += row_h + ui.theme.metrics.spacing;

        let intensity_rect = Rect::new(body.x, y, body.width, row_h);
        Slider::new("Intensity", 0.0, 10.0).show(
            ui,
            id.child("intensity"),
            intensity_rect,
            &mut self.material_intensity,
        );
        y += row_h + ui.theme.metrics.spacing;

        if !self.extra_rows.is_empty() {
            let extra_rect = Rect::new(
                body.x,
                y,
                body.width,
                (body.y + body.height - y).max(0.0),
            );
            PropertyGrid::new(&self.extra_rows).show(ui, id.child("extra"), extra_rect);
        }
    }
}
