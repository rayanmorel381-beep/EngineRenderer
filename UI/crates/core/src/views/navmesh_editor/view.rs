use crate::scene::navmesh::{NavAgent, NavMesh};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct NavmeshEditorView {
    pub grid_width: usize,
    pub grid_depth: usize,
}

impl Default for NavmeshEditorView {
    fn default() -> Self { Self { grid_width: 20, grid_depth: 20 } }
}

impl NavmeshEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, mesh: &mut NavMesh, agents: &mut Vec<NavAgent>) {
        let panel = Panel::new("NavMesh & Agents").with_icon(Icon::Grid);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("navmesh_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let mut gw_f = self.grid_width as f64;
        if Slider::new("Largeur grille", 4.0, 80.0).show(ui, id.child("gw"), Rect::new(lx, y, w, row_h), &mut gw_f) {
            self.grid_width = gw_f as usize;
        }
        y += row_h + sp;
        let mut gd_f = self.grid_depth as f64;
        if Slider::new("Profondeur grille", 4.0, 80.0).show(ui, id.child("gd"), Rect::new(lx, y, w, row_h), &mut gd_f) {
            self.grid_depth = gd_f as usize;
        }
        y += row_h + sp;
        Slider::new("Taille cellule", 0.25, 2.0).show(ui, id.child("cs"), Rect::new(lx, y, w, row_h), &mut mesh.cell_size);
        y += row_h + sp;
        Slider::new("Rayon agent", 0.1, 2.0).show(ui, id.child("ar"), Rect::new(lx, y, w, row_h), &mut mesh.agent_radius);
        y += row_h + sp;
        Slider::new("Hauteur agent", 0.5, 4.0).show(ui, id.child("ah"), Rect::new(lx, y, w, row_h), &mut mesh.agent_height);
        y += row_h + sp;

        let info = format!("Polygones: {}", mesh.polygons.len());
        ui.draw_list.text(Vec2::new(lx, y), &info, p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        if Button::new("Générer grille").with_style(ButtonStyle::Primary)
            .show(ui, id.child("build"), Rect::new(lx, y, w, row_h)).clicked {
            mesh.build_grid(self.grid_width, self.grid_depth);
        }
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Agents IA ({})", agents.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_agent: Option<usize> = None;
        for (i, agent) in agents.iter().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, p.panel, 2.0);
            let arrived = if agent.has_arrived() { "arrivé" } else { "en route" };
            let lbl = format!("Agent {i} [{arrived}] pos({:.1},{:.1},{:.1})", agent.position[0], agent.position[1], agent.position[2]);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &lbl, p.text, m.font_size_small);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("adel{i}")), del_r).clicked {
                remove_agent = Some(i);
            }
            y += row_h + 2.0;
        }
        if let Some(idx) = remove_agent { agents.remove(idx); }

        if Button::new("+ Agent").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("add_agent"), Rect::new(lx, y, w, row_h)).clicked {
            agents.push(NavAgent::new([0.0, 0.0, 0.0]));
        }
    }
}
