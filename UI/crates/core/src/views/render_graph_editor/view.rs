use crate::scene::render_graph::{PassKind, RenderGraph};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct RenderGraphEditorView {
    pub scroll_offset: [f64; 2],
    pub selected_pass: Option<u32>,
    pub kind_idx: usize,
}

impl Default for RenderGraphEditorView {
    fn default() -> Self { Self { scroll_offset: [0.0; 2], selected_pass: None, kind_idx: 0 } }
}

impl RenderGraphEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, graph: &mut RenderGraph) {
        let panel = Panel::new("Render Graph").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("rg_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;

        let (sidebar_r, canvas_r) = body.split_left(200.0);

        let mut sy = sidebar_r.y + pad;
        let slx = sidebar_r.x + pad;
        let sw = sidebar_r.width - pad * 2.0;
        ui.draw_list.text(Vec2::new(slx, sy), &format!("Passes ({})", graph.passes.len()), p.text, m.font_size_normal);
        sy += m.font_size_normal + sp;
        let mut remove_pass: Option<u32> = None;
        for pass in &mut graph.passes {
            let sel = self.selected_pass == Some(pass.id);
            let pr = Rect::new(slx, sy, sw, row_h);
            ui.draw_list.rect(pr, if sel { p.panel_active } else { p.panel }, 2.0);
            let en_col = if pass.enabled { p.success } else { p.text_muted };
            let dot = if pass.enabled { "●" } else { "○" };
            ui.draw_list.text(Vec2::new(slx + 4.0, sy + (row_h - m.font_size_small) * 0.5), dot, en_col, m.font_size_small);
            ui.draw_list.text(Vec2::new(slx + 18.0, sy + (row_h - m.font_size_small) * 0.5), &pass.name, p.text, m.font_size_small);
            let del_r = Rect::new(slx + sw - 20.0, sy + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("pdel{}", pass.id)), del_r).clicked {
                remove_pass = Some(pass.id);
            }
            if ui.is_rect_hovered(pr) { self.selected_pass = Some(pass.id); }
            sy += row_h + 2.0;
        }
        if let Some(pid) = remove_pass { graph.remove_pass(pid); self.selected_pass = None; }

        let kind_lbl = PassKind::ALL[self.kind_idx % PassKind::ALL.len()].label();
        if Button::new(kind_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("kind_cyc"), Rect::new(slx, sy, sw, row_h)).clicked {
            self.kind_idx = (self.kind_idx + 1) % PassKind::ALL.len();
        }
        sy += row_h + 2.0;
        if Button::new("+ Passe").with_style(ButtonStyle::Secondary).show(ui, id.child("add_pass"), Rect::new(slx, sy, sw, row_h)).clicked {
            let kind = PassKind::ALL[self.kind_idx % PassKind::ALL.len()].clone();
            graph.add_pass(kind);
        }

        ui.draw_list.rect(canvas_r, p.viewport_clear, 0.0);
        let pass_w = 160.0;
        let pass_h = 80.0;
        for pass in &graph.passes {
            let nx = canvas_r.x + pass.position[0] + self.scroll_offset[0];
            let ny = canvas_r.y + pass.position[1] + self.scroll_offset[1];
            if nx + pass_w < canvas_r.x || nx > canvas_r.x + canvas_r.width { continue; }
            let pr = Rect::new(nx, ny, pass_w, pass_h);
            let border = if self.selected_pass == Some(pass.id) { p.accent } else { p.border };
            ui.draw_list.rect(pr, p.panel, 4.0);
            ui.draw_list.rect_outline(pr, border, 4.0, 1.5);
            let hdr = Rect::new(nx, ny, pass_w, 20.0);
            let kind_col = pass_kind_color(&pass.kind, p.accent);
            ui.draw_list.rect(hdr, kind_col, 4.0);
            ui.draw_list.text(Vec2::new(nx + 4.0, ny + 3.0), &pass.name, p.text, m.font_size_small);
            let out_str = pass.outputs.join(", ");
            ui.draw_list.text(Vec2::new(nx + 4.0, ny + 26.0), &format!("→ {}", out_str), p.text_muted, m.font_size_small);
        }

        for edge in &graph.edges {
            let from = graph.passes.iter().find(|pp| pp.id == edge.from);
            let to = graph.passes.iter().find(|pp| pp.id == edge.to);
            if let (Some(fp), Some(tp)) = (from, to) {
                let fx = canvas_r.x + fp.position[0] + self.scroll_offset[0] + pass_w;
                let fy = canvas_r.y + fp.position[1] + self.scroll_offset[1] + pass_h * 0.5;
                let tx2 = canvas_r.x + tp.position[0] + self.scroll_offset[0];
                let ty2 = canvas_r.y + tp.position[1] + self.scroll_offset[1] + pass_h * 0.5;
                ui.draw_list.line(Vec2::new(fx, fy), Vec2::new(tx2, ty2), p.accent, 1.5);
                let mx = (fx + tx2) * 0.5;
                let my = (fy + ty2) * 0.5;
                ui.draw_list.text(Vec2::new(mx - 20.0, my - m.font_size_small), &edge.texture, p.text_muted, m.font_size_small);
            }
        }
        let status_y = body.y + body.height - m.font_size_small - sp;
        let status = format!("{} passes | {} arêtes | déf. [{:.0},{:.0}]", graph.passes.len(), graph.edges.len(), self.scroll_offset[0], self.scroll_offset[1]);
        ui.draw_list.line(Vec2::new(lx, status_y - 2.0), Vec2::new(lx + w, status_y - 2.0), p.border, 1.0);
        ui.draw_list.text(Vec2::new(lx, status_y), &status, p.text_muted, m.font_size_small);
    }
}

fn pass_kind_color(kind: &PassKind, fallback: [f64; 4]) -> [f64; 4] {
    match kind {
        PassKind::GBuffer => [0.3, 0.4, 0.7, 1.0],
        PassKind::Shadow => [0.15, 0.15, 0.3, 1.0],
        PassKind::Lighting => [0.8, 0.7, 0.2, 1.0],
        PassKind::AmbientOcclusion => [0.4, 0.4, 0.4, 1.0],
        PassKind::Bloom => [0.9, 0.7, 0.1, 1.0],
        PassKind::ToneMapping => [0.6, 0.3, 0.6, 1.0],
        PassKind::PostProcess => [0.3, 0.6, 0.6, 1.0],
        PassKind::UI => [0.2, 0.6, 0.3, 1.0],
        PassKind::Present => [0.2, 0.7, 0.2, 1.0],
        PassKind::Custom => fallback,
    }
}
