use crate::scene::mesh::{EditMesh, FaceId, SelectMode, VertId};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::{Panel, ToolBar, ToolBarItem};
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, PropertyRow, PropertyGrid, Tab, TabBar};

fn vadd(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn vsub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn vdot(a: [f64; 3], b: [f64; 3]) -> f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn vcross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn vnorm(a: [f64; 3]) -> [f64; 3] {
    let len = (a[0]*a[0]+a[1]*a[1]+a[2]*a[2]).sqrt();
    if len < 1e-12 { [0., 1., 0.] } else { [a[0]/len, a[1]/len, a[2]/len] }
}
fn vscale(a: [f64; 3], t: f64) -> [f64; 3] { [a[0]*t, a[1]*t, a[2]*t] }

struct Cam {
    pos: [f64; 3],
    right: [f64; 3],
    up: [f64; 3],
    forward: [f64; 3],
    fov_scale: f64,
    canvas: Rect,
}

impl Cam {
    fn new(yaw: f64, pitch: f64, dist: f64, target: [f64; 3], canvas: Rect) -> Self {
        let yr = yaw.to_radians(); let pr = pitch.to_radians();
        let pos = [
            target[0]+pr.cos()*yr.sin()*dist,
            target[1]+pr.sin()*dist,
            target[2]+pr.cos()*yr.cos()*dist,
        ];
        let forward = vnorm(vsub(target, pos));
        let world_up = if pitch.abs() > 88.5 { [1., 0., 0.] } else { [0., 1., 0.] };
        let right = vnorm(vcross(forward, world_up));
        let up = vcross(right, forward);
        let fov_scale = (canvas.height*0.5)/(50.0_f64.to_radians()*0.5).tan();
        Self { pos, right, up, forward, fov_scale, canvas }
    }

    fn project(&self, pt: [f64; 3]) -> Option<Vec2> {
        let v = vsub(pt, self.pos);
        let cx = vdot(v, self.right);
        let cy = vdot(v, self.up);
        let cz = vdot(v, self.forward);
        if cz <= 0.05 { return None; }
        Some(Vec2::new(
            self.canvas.x + self.canvas.width*0.5 + cx/cz*self.fov_scale,
            self.canvas.y + self.canvas.height*0.5 - cy/cz*self.fov_scale,
        ))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MeshTool {
    Select,
    Extrude,
    LoopCut,
    Bevel,
    Inset,
    Subdivide,
    Merge,
    FlipNormals,
}

impl Default for MeshTool {
    fn default() -> Self { Self::Select }
}

#[derive(Clone, Debug, Default)]
pub struct PendingOp {
    pub extrude: bool,
    pub extrude_amount: f64,
    pub loop_cut: bool,
    pub loop_cut_t: f64,
    pub bevel: bool,
    pub bevel_amount: f64,
    pub inset: Option<FaceId>,
    pub inset_amount: f64,
    pub subdivide: bool,
    pub merge: bool,
    pub merge_dist: f64,
    pub flip_normals: bool,
}

pub struct MeshEditorView {
    pub active_tool: MeshTool,
    pub orbit_yaw: f64,
    pub orbit_pitch: f64,
    pub orbit_dist: f64,
    pub orbit_target: [f64; 3],
    orbit_active: bool,
    was_left_down: bool,
    was_middle_down: bool,
    was_right_down: bool,
    left_press_pos: Option<(f64, f64)>,
    left_drag_dist: f64,
    pub pending: PendingOp,
    pub extrude_amount: f64,
    pub loop_cut_t: f64,
    pub bevel_amount: f64,
    pub inset_amount: f64,
    pub merge_dist: f64,
    param_tab: usize,
}

impl Default for MeshEditorView {
    fn default() -> Self {
        Self {
            active_tool: MeshTool::Select,
            orbit_yaw: 45.0,
            orbit_pitch: 30.0,
            orbit_dist: 4.0,
            orbit_target: [0.0; 3],
            orbit_active: false,
            was_left_down: false,
            was_middle_down: false,
            was_right_down: false,
            left_press_pos: None,
            left_drag_dist: 0.0,
            pending: PendingOp::default(),
            extrude_amount: 0.5,
            loop_cut_t: 0.5,
            bevel_amount: 0.1,
            inset_amount: 0.2,
            merge_dist: 0.01,
            param_tab: 0,
        }
    }
}

impl MeshEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, mesh: &mut EditMesh) {
        let panel = Panel::new("Mesh Editor").with_icon(Icon::Mesh);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);

        let toolbar_h = ui.theme.metrics.tool_bar_height;
        let tools_rect = Rect::new(body.x, body.y, body.width, toolbar_h);
        let items = [
            ToolBarItem::new(Icon::Filter,   "Select").active(self.active_tool == MeshTool::Select),
            ToolBarItem::new(Icon::Move,     "Extrude (E)").active(self.active_tool == MeshTool::Extrude),
            ToolBarItem::new(Icon::Add,      "Loop Cut (Ctrl+R)").active(self.active_tool == MeshTool::LoopCut),
            ToolBarItem::new(Icon::Scale,    "Bevel (Ctrl+B)").active(self.active_tool == MeshTool::Bevel),
            ToolBarItem::new(Icon::Snap,     "Inset (I)").active(self.active_tool == MeshTool::Inset),
            ToolBarItem::new(Icon::Mesh,     "Subdivide").active(self.active_tool == MeshTool::Subdivide),
            ToolBarItem::new(Icon::Copy,     "Merge").active(self.active_tool == MeshTool::Merge),
            ToolBarItem::new(Icon::Rotate,   "Flip Normals").active(self.active_tool == MeshTool::FlipNormals),
        ];
        if let Some(idx) = ToolBar::new(&items).show(ui, tools_rect) {
            self.active_tool = match idx {
                0 => MeshTool::Select,
                1 => MeshTool::Extrude,
                2 => MeshTool::LoopCut,
                3 => MeshTool::Bevel,
                4 => MeshTool::Inset,
                5 => MeshTool::Subdivide,
                6 => MeshTool::Merge,
                7 => MeshTool::FlipNormals,
                _ => self.active_tool,
            };
        }

        let mode_bar_h = toolbar_h;
        let mode_rect = Rect::new(body.x, body.y + toolbar_h + 2.0, body.width, mode_bar_h);
        self.show_select_mode_bar(ui, mode_rect, mesh);

        let param_h = 90.0;
        let param_rect = Rect::new(
            body.x,
            body.y + toolbar_h * 2.0 + 4.0,
            body.width,
            param_h,
        );
        self.show_params_panel(ui, param_rect, mesh);

        let viewport_y = body.y + toolbar_h * 2.0 + 4.0 + param_h + 4.0;
        let viewport_h = (body.y + body.height - viewport_y).max(40.0);
        let canvas = Rect::new(body.x, viewport_y, body.width, viewport_h);

        let cam = Cam::new(
            self.orbit_yaw, self.orbit_pitch, self.orbit_dist,
            self.orbit_target, canvas,
        );

        ui.draw_list.push_clip(canvas);
        ui.draw_list.rect(canvas, [0.06, 0.06, 0.08, 1.0], 0.0);
        self.draw_mesh(ui, &cam, mesh);
        ui.draw_list.pop_clip();

        self.process_viewport_input(ui, canvas, &cam, mesh);
    }

    fn show_select_mode_bar(&mut self, ui: &mut UiContext, rect: Rect, mesh: &mut EditMesh) {
        ui.draw_list.rect(rect, ui.theme.palette.panel, ui.theme.metrics.corner_radius_small);
        let pad = 4.0;
        let btn_w = (rect.width / 3.0 - pad).max(20.0);
        let modes = [
            (SelectMode::Vertex, "Verts", WidgetId::hash_str("me_sel_vert")),
            (SelectMode::Edge,   "Edges", WidgetId::hash_str("me_sel_edge")),
            (SelectMode::Face,   "Faces", WidgetId::hash_str("me_sel_face")),
        ];
        for (i, (mode, label, id)) in modes.into_iter().enumerate() {
            let x = rect.x + i as f64 * (btn_w + pad) + pad;
            let btn_rect = Rect::new(x, rect.y + 2.0, btn_w, rect.height - 4.0);
            let active = mesh.select_mode == mode;
            let style = if active {
                crate::ui::widgets::ButtonStyle::Primary
            } else {
                crate::ui::widgets::ButtonStyle::Secondary
            };
            let r = Button::new(label).with_style(style).show(ui, id, btn_rect);
            if r.clicked {
                mesh.select_mode = mode;
                mesh.deselect_all();
            }
        }
    }

    fn show_params_panel(&mut self, ui: &mut UiContext, rect: Rect, mesh: &mut EditMesh) {
        ui.draw_list.rect(rect, ui.theme.palette.panel, ui.theme.metrics.corner_radius_small);
        let row_h = ui.theme.metrics.row_height;
        let pad = 4.0;
        let inner = Rect::new(rect.x + pad, rect.y + pad, rect.width - pad * 2.0, rect.height - pad * 2.0);

        match self.active_tool {
            MeshTool::Select => {
                let tabs = [Tab::new("All"), Tab::new("None"), Tab::new("Invert")];
                let tab_rect = Rect::new(inner.x, inner.y, inner.width, row_h);
                TabBar::new(&tabs).show(ui, WidgetId::hash_str("me_sel_tabs"), tab_rect, &mut self.param_tab);
                match self.param_tab {
                    0 => { let r = Button::new("Select All").show(ui, WidgetId::hash_str("me_selall"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h)); if r.clicked { mesh.select_all(); } }
                    1 => { let r = Button::new("Deselect All").show(ui, WidgetId::hash_str("me_selnone"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h)); if r.clicked { mesh.deselect_all(); } }
                    _ => {
                        let r = Button::new("Invert Selection").show(ui, WidgetId::hash_str("me_selinv"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h));
                        if r.clicked {
                            match mesh.select_mode {
                                SelectMode::Vertex => { for v in &mut mesh.verts { v.selected = !v.selected; } }
                                SelectMode::Edge => { for e in &mut mesh.edges { e.selected = !e.selected; } }
                                SelectMode::Face => { for f in &mut mesh.faces { f.selected = !f.selected; } }
                            }
                        }
                    }
                }
            }
            MeshTool::Extrude => {
                let rows = [PropertyRow::new("Offset", format!("{:.3}", self.extrude_amount))];
                PropertyGrid::new(&rows).show(ui, WidgetId::hash_str("me_ext_rows"), Rect::new(inner.x, inner.y, inner.width, row_h));
                use crate::ui::widgets::Slider;
                Slider::new("Amount", 0.01, 5.0).show(ui, WidgetId::hash_str("me_ext_slider"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h), &mut self.extrude_amount);
                let r = Button::new("Apply Extrude").show(ui, WidgetId::hash_str("me_ext_apply"), Rect::new(inner.x, inner.y + row_h * 2.0 + 8.0, inner.width, row_h));
                if r.clicked { self.pending.extrude = true; self.pending.extrude_amount = self.extrude_amount; }
            }
            MeshTool::LoopCut => {
                use crate::ui::widgets::Slider;
                Slider::new("Position", 0.0, 1.0).show(ui, WidgetId::hash_str("me_lc_slider"), Rect::new(inner.x, inner.y, inner.width, row_h), &mut self.loop_cut_t);
                let r = Button::new("Apply Loop Cut").show(ui, WidgetId::hash_str("me_lc_apply"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h));
                if r.clicked { self.pending.loop_cut = true; self.pending.loop_cut_t = self.loop_cut_t; }
            }
            MeshTool::Bevel => {
                use crate::ui::widgets::Slider;
                Slider::new("Amount", 0.01, 2.0).show(ui, WidgetId::hash_str("me_bev_slider"), Rect::new(inner.x, inner.y, inner.width, row_h), &mut self.bevel_amount);
                let r = Button::new("Apply Bevel").show(ui, WidgetId::hash_str("me_bev_apply"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h));
                if r.clicked { self.pending.bevel = true; self.pending.bevel_amount = self.bevel_amount; }
            }
            MeshTool::Inset => {
                use crate::ui::widgets::Slider;
                Slider::new("Amount", 0.01, 0.95).show(ui, WidgetId::hash_str("me_ins_slider"), Rect::new(inner.x, inner.y, inner.width, row_h), &mut self.inset_amount);
                let r = Button::new("Apply Inset").show(ui, WidgetId::hash_str("me_ins_apply"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h));
                if r.clicked {
                    let sel = mesh.selected_faces();
                    if let Some(fid) = sel.into_iter().next() {
                        self.pending.inset = Some(fid);
                        self.pending.inset_amount = self.inset_amount;
                    }
                }
            }
            MeshTool::Subdivide => {
                let r = Button::new("Subdivide Once").show(ui, WidgetId::hash_str("me_sub_apply"), Rect::new(inner.x, inner.y, inner.width, row_h));
                if r.clicked { self.pending.subdivide = true; }
            }
            MeshTool::Merge => {
                use crate::ui::widgets::Slider;
                Slider::new("Threshold", 0.0001, 0.5).show(ui, WidgetId::hash_str("me_mrg_slider"), Rect::new(inner.x, inner.y, inner.width, row_h), &mut self.merge_dist);
                let r = Button::new("Merge by Distance").show(ui, WidgetId::hash_str("me_mrg_apply"), Rect::new(inner.x, inner.y + row_h + 4.0, inner.width, row_h));
                if r.clicked { self.pending.merge = true; self.pending.merge_dist = self.merge_dist; }
            }
            MeshTool::FlipNormals => {
                let r = Button::new("Flip Normals").show(ui, WidgetId::hash_str("me_flip_apply"), Rect::new(inner.x, inner.y, inner.width, row_h));
                if r.clicked { self.pending.flip_normals = true; }
            }
        }

        let stat_y = inner.y + inner.height - ui.theme.metrics.font_size_small - 2.0;
        let stat = format!(
            "{} verts  {} edges  {} faces  {} sel",
            mesh.verts.len(), mesh.edges.len(), mesh.faces.len(),
            match mesh.select_mode {
                SelectMode::Vertex => mesh.verts.iter().filter(|v| v.selected).count(),
                SelectMode::Edge   => mesh.edges.iter().filter(|e| e.selected).count(),
                SelectMode::Face   => mesh.faces.iter().filter(|f| f.selected).count(),
            }
        );
        ui.draw_list.text(
            Vec2::new(inner.x, stat_y),
            stat,
            ui.theme.palette.text_muted,
            ui.theme.metrics.font_size_small,
        );
    }

    fn draw_mesh(&self, ui: &mut UiContext, cam: &Cam, mesh: &EditMesh) {
        let col_edge      = [0.45f64, 0.60, 0.75, 1.0];
        let col_edge_sel  = [1.0,  0.72, 0.1,  1.0];
        let col_vert      = [0.55, 0.75, 1.0,  1.0];
        let col_vert_sel  = [1.0,  0.85, 0.2,  1.0];
        let col_face      = [0.15, 0.4,  0.7,  0.18];
        let col_face_sel  = [1.0,  0.65, 0.1,  0.30];

        for face in &mesh.faces {
            let col = if face.selected { col_face_sel } else { col_face };
            let n = face.verts.len();
            if n < 3 { continue; }
            let center_pos = {
                let sum = face.verts.iter().fold([0.0f64; 3], |acc, vid| {
                    vadd(acc, mesh.vert_pos(*vid).unwrap_or([0.0; 3]))
                });
                vscale(sum, 1.0 / n as f64)
            };
            for i in 0..n {
                let a = mesh.vert_pos(face.verts[i]).unwrap_or([0.0; 3]);
                let b = mesh.vert_pos(face.verts[(i + 1) % n]).unwrap_or([0.0; 3]);
                let c = center_pos;
                if let (Some(sa), Some(sb), Some(sc)) = (cam.project(a), cam.project(b), cam.project(c)) {
                    let mid_ab = Vec2::new((sa.x+sb.x)*0.5, (sa.y+sb.y)*0.5);
                    let mid_bc = Vec2::new((sb.x+sc.x)*0.5, (sb.y+sc.y)*0.5);
                    let mid_ca = Vec2::new((sc.x+sa.x)*0.5, (sc.y+sa.y)*0.5);
                    ui.draw_list.line(sa, mid_ab, col, 1.0);
                    ui.draw_list.line(mid_ab, sb, col, 1.0);
                    ui.draw_list.line(sb, mid_bc, col, 1.0);
                    ui.draw_list.line(mid_bc, sc, col, 1.0);
                    ui.draw_list.line(sc, mid_ca, col, 1.0);
                    ui.draw_list.line(mid_ca, sa, col, 1.0);
                }
            }
        }

        for edge in &mesh.edges {
            let col = if edge.selected { col_edge_sel } else { col_edge };
            let thick = if edge.selected { 2.5 } else { 1.0 };
            let pa = mesh.vert_pos(edge.verts[0]).unwrap_or([0.0; 3]);
            let pb = mesh.vert_pos(edge.verts[1]).unwrap_or([0.0; 3]);
            if let (Some(sa), Some(sb)) = (cam.project(pa), cam.project(pb)) {
                ui.draw_list.line(sa, sb, col, thick);
            }
        }

        for vert in &mesh.verts {
            let col = if vert.selected { col_vert_sel } else { col_vert };
            let r = if vert.selected { 4.5 } else { 2.5 };
            if let Some(sp) = cam.project(vert.pos) {
                ui.draw_list.rect(
                    Rect::new(sp.x - r, sp.y - r, r * 2.0, r * 2.0),
                    col,
                    r * 0.4,
                );
            }
        }
    }

    fn process_viewport_input(&mut self, ui: &mut UiContext, canvas: Rect, cam: &Cam, mesh: &mut EditMesh) {
        let mx = ui.input.pointer.x;
        let my = ui.input.pointer.y;
        let in_canvas = canvas.contains(Vec2::new(mx, my));
        let (dx, dy) = ui.input.pointer.delta();
        let left   = ui.input.pointer.left_down;
        let middle = ui.input.pointer.middle_down;
        let right  = ui.input.pointer.right_down;
        let scroll = ui.input.pointer.scroll_y;

        let just_left          = left  && !self.was_left_down;
        let just_released_left = !left && self.was_left_down;

        if in_canvas && scroll.abs() > 1e-6 {
            let f = if scroll > 0.0 { 0.85 } else { 1.15 };
            self.orbit_dist = (self.orbit_dist * f).clamp(0.05, 200.0);
        }

        if just_left && in_canvas {
            self.left_press_pos = Some((mx, my));
            self.left_drag_dist = 0.0;
            self.orbit_active = true;
        }

        if left && self.orbit_active {
            if let Some(pp) = self.left_press_pos {
                self.left_drag_dist = ((mx-pp.0).powi(2)+(my-pp.1).powi(2)).sqrt();
            }
            if self.left_drag_dist > 4.0 {
                self.orbit_yaw   = (self.orbit_yaw + dx*0.5) % 360.0;
                self.orbit_pitch = (self.orbit_pitch - dy*0.5).clamp(-89.0, 89.0);
            }
        }

        if right {
            self.orbit_yaw   = (self.orbit_yaw + dx*0.5) % 360.0;
            self.orbit_pitch = (self.orbit_pitch - dy*0.5).clamp(-89.0, 89.0);
        }

        if middle {
            let sp = self.orbit_dist * 0.003;
            self.orbit_target[0] -= (cam.right[0]*dx - cam.up[0]*dy)*sp;
            self.orbit_target[1] -= (cam.right[1]*dx - cam.up[1]*dy)*sp;
            self.orbit_target[2] -= (cam.right[2]*dx - cam.up[2]*dy)*sp;
        }

        if just_released_left {
            if self.orbit_active && self.left_drag_dist < 4.0 {
                if let Some(pp) = self.left_press_pos {
                    if canvas.contains(Vec2::new(pp.0, pp.1)) {
                        self.pick_element(cam, mesh, pp.0, pp.1);
                    }
                }
            }
            self.orbit_active   = false;
            self.left_press_pos = None;
            self.left_drag_dist = 0.0;
        }

        self.was_left_down   = left;
        self.was_middle_down = middle;
        self.was_right_down  = right;
    }

    fn pick_element(&self, cam: &Cam, mesh: &mut EditMesh, mx: f64, my: f64) {
        let thresh = 12.0;
        match mesh.select_mode {
            SelectMode::Vertex => {
                let mut best_d = thresh;
                let mut best_id: Option<VertId> = None;
                for vert in &mesh.verts {
                    if let Some(sp) = cam.project(vert.pos) {
                        let d = ((sp.x-mx).powi(2)+(sp.y-my).powi(2)).sqrt();
                        if d < best_d { best_d = d; best_id = Some(vert.id); }
                    }
                }
                if let Some(id) = best_id {
                    for v in &mut mesh.verts {
                        if v.id == id { v.selected = !v.selected; }
                    }
                } else {
                    mesh.deselect_all();
                }
            }
            SelectMode::Edge => {
                let mut best_d = thresh;
                let mut best_id: Option<crate::scene::mesh::EdgeId> = None;
                for edge in &mesh.edges {
                    let pa = mesh.vert_pos(edge.verts[0]).unwrap_or([0.0; 3]);
                    let pb = mesh.vert_pos(edge.verts[1]).unwrap_or([0.0; 3]);
                    if let (Some(sa), Some(sb)) = (cam.project(pa), cam.project(pb)) {
                        let d = point_segment_dist(Vec2::new(mx, my), sa, sb);
                        if d < best_d { best_d = d; best_id = Some(edge.id); }
                    }
                }
                if let Some(id) = best_id {
                    for e in &mut mesh.edges {
                        if e.id == id { e.selected = !e.selected; }
                    }
                } else {
                    mesh.deselect_all();
                }
            }
            SelectMode::Face => {
                let mut best_d = thresh * 2.0;
                let mut best_id: Option<FaceId> = None;
                for face in &mesh.faces {
                    let n = face.verts.len();
                    if n == 0 { continue; }
                    let sum = face.verts.iter().fold([0.0f64; 3], |acc, vid| {
                        vadd(acc, mesh.vert_pos(*vid).unwrap_or([0.0; 3]))
                    });
                    let center = vscale(sum, 1.0 / n as f64);
                    if let Some(sc) = cam.project(center) {
                        let d = ((sc.x-mx).powi(2)+(sc.y-my).powi(2)).sqrt();
                        if d < best_d { best_d = d; best_id = Some(face.id); }
                    }
                }
                if let Some(id) = best_id {
                    for f in &mut mesh.faces {
                        if f.id == id { f.selected = !f.selected; }
                    }
                } else {
                    mesh.deselect_all();
                }
            }
        }
    }
}

fn point_segment_dist(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    let ap = Vec2::new(p.x - a.x, p.y - a.y);
    let ab_sq = ab.x*ab.x + ab.y*ab.y;
    if ab_sq < 1e-12 {
        return (ap.x*ap.x + ap.y*ap.y).sqrt();
    }
    let t = ((ap.x*ab.x + ap.y*ab.y) / ab_sq).clamp(0.0, 1.0);
    let proj = Vec2::new(a.x + ab.x*t, a.y + ab.y*t);
    ((p.x-proj.x).powi(2) + (p.y-proj.y).powi(2)).sqrt()
}
