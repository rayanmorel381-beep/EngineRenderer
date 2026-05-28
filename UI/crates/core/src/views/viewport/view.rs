use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::{Panel, ToolBar, ToolBarItem};
use crate::ui::style::icons::Icon;
use crate::{ObjectId, ObjectKind, PrimitiveKind, Scene};

fn vadd(a: [f64;3], b: [f64;3]) -> [f64;3] { [a[0]+b[0],a[1]+b[1],a[2]+b[2]] }
fn vsub(a: [f64;3], b: [f64;3]) -> [f64;3] { [a[0]-b[0],a[1]-b[1],a[2]-b[2]] }
fn vscale(a: [f64;3], t: f64) -> [f64;3] { [a[0]*t,a[1]*t,a[2]*t] }
fn vdot(a: [f64;3], b: [f64;3]) -> f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn vcross(a: [f64;3], b: [f64;3]) -> [f64;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn vnorm(a: [f64;3]) -> [f64;3] {
    let len = (a[0]*a[0]+a[1]*a[1]+a[2]*a[2]).sqrt();
    if len < 1e-12 { [0.,1.,0.] } else { [a[0]/len,a[1]/len,a[2]/len] }
}
fn rotate_euler(pt: [f64;3], rx: f64, ry: f64, rz: f64) -> [f64;3] {
    let (sx,cx) = (rx.to_radians().sin(), rx.to_radians().cos());
    let (_sy,_cy) = (ry.to_radians().sin(), ry.to_radians().cos());
    let (sz,cz) = (rz.to_radians().sin(), rz.to_radians().cos());
    let sy = _sy; let cy = _cy;
    let x1=pt[0]*cz-pt[1]*sz; let y1=pt[0]*sz+pt[1]*cz; let z1=pt[2];
    let x2=x1; let y2=y1*cx-z1*sx; let z2=y1*sx+z1*cx;
    [x2*cy+z2*sy, y2, -x2*sy+z2*cy]
}

struct Camera {
    pos: [f64;3], right: [f64;3], up: [f64;3], forward: [f64;3],
    fov_scale: f64, canvas: Rect,
}
impl Camera {
    fn from_orbit(target: [f64;3], yaw: f64, pitch: f64, dist: f64, canvas: Rect) -> Self {
        let yr = yaw.to_radians(); let pr = pitch.to_radians();
        let pos = [
            target[0]+pr.cos()*yr.sin()*dist,
            target[1]+pr.sin()*dist,
            target[2]+pr.cos()*yr.cos()*dist,
        ];
        let forward = vnorm(vsub(target, pos));
        let world_up = if pitch.abs() > 88.5 { [1.,0.,0.] } else { [0.,1.,0.] };
        let right = vnorm(vcross(forward, world_up));
        let up = vcross(right, forward);
        let fov_scale = (canvas.height*0.5)/(50.0_f64.to_radians()*0.5).tan();
        Self { pos, right, up, forward, fov_scale, canvas }
    }
    fn project(&self, pt: [f64;3]) -> Option<Vec2> {
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

type Edges = Vec<([f64;3],[f64;3])>;

fn cube_edges() -> Edges {
    let v: [[f64;3];8] = [
        [-0.5,-0.5,-0.5],[0.5,-0.5,-0.5],[0.5,0.5,-0.5],[-0.5,0.5,-0.5],
        [-0.5,-0.5, 0.5],[0.5,-0.5, 0.5],[0.5,0.5, 0.5],[-0.5,0.5, 0.5],
    ];
    vec![(v[0],v[1]),(v[1],v[2]),(v[2],v[3]),(v[3],v[0]),
         (v[4],v[5]),(v[5],v[6]),(v[6],v[7]),(v[7],v[4]),
         (v[0],v[4]),(v[1],v[5]),(v[2],v[6]),(v[3],v[7])]
}
fn plane_edges() -> Edges {
    vec![([-0.5,0.,-0.5],[0.5,0.,-0.5]),([0.5,0.,-0.5],[0.5,0.,0.5]),
         ([0.5,0.,0.5],[-0.5,0.,0.5]),([-0.5,0.,0.5],[-0.5,0.,-0.5]),
         ([0.,0.,-0.5],[0.,0.,0.5]),([-0.5,0.,0.],[0.5,0.,0.])]
}
fn circle_ring(r: f64, y: f64, n: usize) -> Edges {
    (0..n).map(|i| {
        let a0 = i as f64/n as f64*std::f64::consts::TAU;
        let a1 = (i+1) as f64/n as f64*std::f64::consts::TAU;
        ([r*a0.cos(),y,r*a0.sin()],[r*a1.cos(),y,r*a1.sin()])
    }).collect()
}
fn cylinder_edges() -> Edges {
    let n = 16;
    let mut e = circle_ring(0.5,-0.5,n);
    e.extend(circle_ring(0.5,0.5,n));
    for i in [0,4,8,12usize] {
        let a = i as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.5*a.cos(),-0.5,0.5*a.sin()],[0.5*a.cos(),0.5,0.5*a.sin()]));
    }
    e
}
fn cone_edges() -> Edges {
    let n = 16;
    let mut e = circle_ring(0.5,-0.5,n);
    for i in [0,4,8,12usize] {
        let a = i as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.5*a.cos(),-0.5,0.5*a.sin()],[0.,0.5,0.]));
    }
    e
}
fn sphere_edges() -> Edges {
    let n = 16; let rings = 6;
    let mut e = Vec::new();
    for r in 1..rings {
        let lat = (r as f64/rings as f64-0.5)*std::f64::consts::PI;
        e.extend(circle_ring(lat.cos()*0.5, lat.sin()*0.5, n));
    }
    for l in 0..6 {
        let lng = l as f64/6.*std::f64::consts::TAU;
        for r in 0..rings {
            let lat0 = (r as f64/rings as f64-0.5)*std::f64::consts::PI;
            let lat1 = ((r+1) as f64/rings as f64-0.5)*std::f64::consts::PI;
            e.push(([lat0.cos()*0.5*lng.cos(),lat0.sin()*0.5,lat0.cos()*0.5*lng.sin()],
                    [lat1.cos()*0.5*lng.cos(),lat1.sin()*0.5,lat1.cos()*0.5*lng.sin()]));
        }
    }
    e
}
fn torus_edges() -> Edges {
    let (mr,nr,nm,nn) = (0.38,0.14,20,10);
    let mut e = Vec::new();
    for i in 0..nm {
        let a0 = i as f64/nm as f64*std::f64::consts::TAU;
        let a1 = (i+1) as f64/nm as f64*std::f64::consts::TAU;
        e.push(([mr*a0.cos(),0.,mr*a0.sin()],[mr*a1.cos(),0.,mr*a1.sin()]));
        for j in 0..nn {
            let b0 = j as f64/nn as f64*std::f64::consts::TAU;
            let b1 = (j+1) as f64/nn as f64*std::f64::consts::TAU;
            let r0 = mr+nr*b0.cos(); let r1 = mr+nr*b1.cos();
            e.push(([r0*a0.cos(),nr*b0.sin(),r0*a0.sin()],[r1*a0.cos(),nr*b1.sin(),r1*a0.sin()]));
        }
    }
    e
}
fn icosphere_edges() -> Edges {
    let phi = (1.+5.0_f64.sqrt())/2.;
    let n = (1.+phi*phi).sqrt();
    let s = 0.5/n; let p = phi*s;
    let v: [[f64;3];12] = [
        [0.,s,p],[0.,-s,p],[0.,s,-p],[0.,-s,-p],
        [s,p,0.],[-s,p,0.],[s,-p,0.],[-s,-p,0.],
        [p,0.,s],[-p,0.,s],[p,0.,-s],[-p,0.,-s],
    ];
    vec![(v[0],v[1]),(v[0],v[4]),(v[0],v[5]),(v[0],v[8]),(v[0],v[9]),
         (v[1],v[6]),(v[1],v[7]),(v[1],v[8]),(v[1],v[9]),
         (v[2],v[3]),(v[2],v[4]),(v[2],v[5]),(v[2],v[10]),(v[2],v[11]),
         (v[3],v[6]),(v[3],v[7]),(v[3],v[10]),(v[3],v[11]),
         (v[4],v[5]),(v[4],v[8]),(v[4],v[10]),
         (v[5],v[9]),(v[5],v[11]),
         (v[6],v[7]),(v[6],v[8]),(v[6],v[10]),
         (v[7],v[9]),(v[7],v[11]),
         (v[8],v[10]),(v[9],v[11])]
}
fn capsule_edges() -> Edges {
    let n = 16;
    let mut e = circle_ring(0.5,0.,n);
    e.extend(circle_ring(0.5,0.5,n));
    e.extend(circle_ring(0.5,-0.5,n));
    for i in [0,4,8,12usize] {
        let a = i as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.5*a.cos(),-0.5,0.5*a.sin()],[0.5*a.cos(),0.5,0.5*a.sin()]));
    }
    for i in 0..n/2 {
        let a0 = i as f64/n as f64*std::f64::consts::TAU;
        let a1 = (i+1) as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.5*a0.cos(),0.5+0.5*a0.sin(),0.],[0.5*a1.cos(),0.5+0.5*a1.sin(),0.]));
        e.push(([0.5*a0.cos(),-0.5-0.5*a0.sin(),0.],[0.5*a1.cos(),-0.5-0.5*a1.sin(),0.]));
    }
    e
}
fn hypercube4d_edges() -> Edges {
    let outer: [[f64;3];8] = [
        [-0.5,-0.5,-0.5],[0.5,-0.5,-0.5],[0.5,0.5,-0.5],[-0.5,0.5,-0.5],
        [-0.5,-0.5, 0.5],[0.5,-0.5, 0.5],[0.5,0.5, 0.5],[-0.5,0.5, 0.5],
    ];
    let inner: [[f64;3];8] = outer.map(|v| [v[0]*0.5,v[1]*0.5,v[2]*0.5]);
    let mut e = cube_edges();
    e.extend([(inner[0],inner[1]),(inner[1],inner[2]),(inner[2],inner[3]),(inner[3],inner[0]),
              (inner[4],inner[5]),(inner[5],inner[6]),(inner[6],inner[7]),(inner[7],inner[4]),
              (inner[0],inner[4]),(inner[1],inner[5]),(inner[2],inner[6]),(inner[3],inner[7])]);
    for i in 0..8 { e.push((outer[i],inner[i])); }
    e
}
fn simplex4d_edges() -> Edges {
    let v: [[f64;3];5] = [[0.,0.5,0.],[0.47,-0.17,0.],[-0.47,-0.17,0.],[0.,0.,0.47],[0.,0.,-0.47]];
    let mut e = Vec::new();
    for i in 0..5 { for j in (i+1)..5 { e.push((v[i],v[j])); } }
    e
}
fn camera_edges() -> Edges {
    vec![([-0.3,-0.2,-0.3],[0.3,-0.2,-0.3]),([0.3,-0.2,-0.3],[0.3,0.2,-0.3]),
         ([0.3,0.2,-0.3],[-0.3,0.2,-0.3]),([-0.3,0.2,-0.3],[-0.3,-0.2,-0.3]),
         ([-0.3,-0.2,0.3],[0.3,-0.2,0.3]),([0.3,-0.2,0.3],[0.3,0.2,0.3]),
         ([0.3,0.2,0.3],[-0.3,0.2,0.3]),([-0.3,0.2,0.3],[-0.3,-0.2,0.3]),
         ([-0.3,-0.2,-0.3],[-0.3,-0.2,0.3]),([0.3,-0.2,-0.3],[0.3,-0.2,0.3]),
         ([0.3,0.2,-0.3],[0.3,0.2,0.3]),([-0.3,0.2,-0.3],[-0.3,0.2,0.3]),
         ([0.,0.,-0.3],[-0.15,-0.15,-0.6]),([0.,0.,-0.3],[0.15,-0.15,-0.6]),
         ([0.,0.,-0.3],[0.15,0.15,-0.6]),([0.,0.,-0.3],[-0.15,0.15,-0.6]),
         ([-0.15,-0.15,-0.6],[0.15,-0.15,-0.6]),([0.15,-0.15,-0.6],[0.15,0.15,-0.6]),
         ([0.15,0.15,-0.6],[-0.15,0.15,-0.6]),([-0.15,0.15,-0.6],[-0.15,-0.15,-0.6])]
}
fn light_point_edges() -> Edges {
    let r = 0.3; let n = 16;
    let mut e = circle_ring(r, 0., n);
    for i in 0..n {
        let a0 = i as f64/n as f64*std::f64::consts::TAU;
        let a1 = (i+1) as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.,r*a0.sin(),r*a0.cos()],[0.,r*a1.sin(),r*a1.cos()]));
    }
    for i in 0..6 {
        let a = i as f64/6.*std::f64::consts::TAU;
        e.push(([0.,0.,0.],[r*1.7*a.cos(),r*1.7*a.sin(),0.]));
    }
    e
}
fn light_dir_edges() -> Edges {
    let mut e = Vec::new();
    for i in 0..5 {
        let x = (i as f64-2.)*0.2;
        e.push(([x,0.4,0.],[x,-0.1,0.]));
        e.push(([x-0.08,-0.1,0.],[x,-0.4,0.]));
        e.push(([x+0.08,-0.1,0.],[x,-0.4,0.]));
    }
    e
}
fn light_spot_edges() -> Edges {
    let n = 16;
    let mut e = circle_ring(0.05,0.,n);
    e.extend(circle_ring(0.4,-0.6,n));
    for i in [0,4,8,12usize] {
        let a = i as f64/n as f64*std::f64::consts::TAU;
        e.push(([0.05*a.cos(),0.,0.05*a.sin()],[0.4*a.cos(),-0.6,0.4*a.sin()]));
    }
    e
}
fn empty_edges() -> Edges {
    let r = 0.25;
    vec![([-r,0.,0.],[r,0.,0.]),([0.,-r,0.],[0.,r,0.]),([0.,0.,-r],[0.,0.,r])]
}

fn object_edges(kind: &ObjectKind) -> Edges {
    match kind {
        ObjectKind::Primitive(PrimitiveKind::Cube)            => cube_edges(),
        ObjectKind::Primitive(PrimitiveKind::Sphere)          => sphere_edges(),
        ObjectKind::Primitive(PrimitiveKind::Plane)           => plane_edges(),
        ObjectKind::Primitive(PrimitiveKind::Cylinder)        => cylinder_edges(),
        ObjectKind::Primitive(PrimitiveKind::Cone)            => cone_edges(),
        ObjectKind::Primitive(PrimitiveKind::Torus)           => torus_edges(),
        ObjectKind::Primitive(PrimitiveKind::Icosphere)       => icosphere_edges(),
        ObjectKind::Primitive(PrimitiveKind::Capsule)         => capsule_edges(),
        ObjectKind::Primitive(PrimitiveKind::Hypercube4D)     => hypercube4d_edges(),
        ObjectKind::Primitive(PrimitiveKind::Simplex4D)       => simplex4d_edges(),
        ObjectKind::Primitive(PrimitiveKind::Camera)          => camera_edges(),
        ObjectKind::Primitive(PrimitiveKind::DirectionalLight)=> light_dir_edges(),
        ObjectKind::Primitive(PrimitiveKind::PointLight)      => light_point_edges(),
        ObjectKind::Primitive(PrimitiveKind::SpotLight)       => light_spot_edges(),
        ObjectKind::Primitive(PrimitiveKind::Empty)           => empty_edges(),
        ObjectKind::Mesh { .. }                               => cube_edges(),
    }
}

fn object_color(kind: &ObjectKind, selected: bool) -> [f64;4] {
    if selected { return [1.0,0.75,0.0,1.0]; }
    match kind {
        ObjectKind::Primitive(PrimitiveKind::Camera) => [0.26,0.65,0.98,0.9],
        ObjectKind::Primitive(
            PrimitiveKind::DirectionalLight|PrimitiveKind::PointLight|PrimitiveKind::SpotLight
        ) => [0.98,0.88,0.26,0.9],
        ObjectKind::Primitive(PrimitiveKind::Empty) => [0.55,0.60,0.72,0.7],
        ObjectKind::Primitive(PrimitiveKind::Hypercube4D|PrimitiveKind::Simplex4D) => [0.76,0.26,0.98,0.9],
        ObjectKind::Primitive(_)|ObjectKind::Mesh{..} => [0.35,0.85,0.52,0.9],
    }
}

#[derive(Clone,Debug)]
struct GizmoDrag {
    axis: u8,
    start_mouse: (f64,f64),
    start_pos: [f64;3],
    axis_screen_dir: (f64,f64),
}

fn circle_on_plane(center: [f64;3], normal: [f64;3], radius: f64, angle: f64) -> [f64;3] {
    let n = vnorm(normal);
    let ref_up = if n[1].abs() < 0.9 { [0.,1.,0.] } else { [1.,0.,0.] };
    let u = vnorm(vcross(ref_up, n));
    let v = vcross(n, u);
    vadd(center, vadd(vscale(u, radius*angle.cos()), vscale(v, radius*angle.sin())))
}

pub struct ViewportView {
    pub framebuffer: Vec<u8>,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub gizmo_mode: u32,
    pub show_grid: bool,
    pub show_stats_overlay: bool,
    pub orbit_yaw: f64,
    pub orbit_pitch: f64,
    pub orbit_dist: f64,
    pub orbit_target: [f64;3],
    orbit_active: bool,
    pan_active: bool,
    was_left_down: bool,
    was_middle_down: bool,
    was_right_down: bool,
    left_press_pos: Option<(f64,f64)>,
    left_drag_dist: f64,
    gizmo_drag: Option<GizmoDrag>,
    pub pending_select: Option<Option<ObjectId>>,
    pub pending_move: Option<(ObjectId,[f64;3])>,
    pub transform_commit: Option<(ObjectId,[f64;3],[f64;3])>,
}

impl Default for ViewportView {
    fn default() -> Self {
        Self {
            framebuffer: Vec::new(),
            framebuffer_width: 0, framebuffer_height: 0,
            gizmo_mode: 0,
            show_grid: true, show_stats_overlay: true,
            orbit_yaw: 45., orbit_pitch: 30., orbit_dist: 12.,
            orbit_target: [0.;3],
            orbit_active: false, pan_active: false,
            was_left_down: false, was_middle_down: false, was_right_down: false,
            left_press_pos: None, left_drag_dist: 0.,
            gizmo_drag: None,
            pending_select: None, pending_move: None, transform_commit: None,
        }
    }
}

impl ViewportView {
    pub fn new() -> Self { Self::default() }

    pub fn set_framebuffer(&mut self, pixels: Vec<u8>, width: u32, height: u32) {
        self.framebuffer = pixels;
        self.framebuffer_width = width;
        self.framebuffer_height = height;
    }

    pub fn frame_camera_on(&mut self, center: [f64;3], radius: f64) {
        self.orbit_target = center;
        self.orbit_dist = (radius*3.).max(2.);
    }

    pub fn show(
        &mut self, ui: &mut UiContext, rect: Rect, scene: &Scene, selected: Option<ObjectId>,
    ) {
        let panel = Panel::new("Viewport").with_icon(Icon::Camera);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);

        let toolbar_rect = Rect::new(body.x, body.y, body.width, ui.theme.metrics.tool_bar_height);
        let items = [
            ToolBarItem::new(Icon::Move,   "Translate (G)").active(self.gizmo_mode == 0),
            ToolBarItem::new(Icon::Rotate, "Rotate (R)").active(self.gizmo_mode == 1),
            ToolBarItem::new(Icon::Scale,  "Scale (S)").active(self.gizmo_mode == 2),
            ToolBarItem::new(Icon::Grid,   "Toggle Grid").active(self.show_grid),
        ];
        if let Some(idx) = ToolBar::new(&items).show(ui, toolbar_rect) {
            match idx {
                0..=2 => self.gizmo_mode = idx as u32,
                3     => self.show_grid = !self.show_grid,
                _     => {}
            }
        }

        let canvas = Rect::new(
            body.x, body.y + ui.theme.metrics.tool_bar_height,
            body.width, (body.height - ui.theme.metrics.tool_bar_height).max(0.),
        );
        ui.draw_list.rect(canvas, ui.theme.palette.viewport_clear, 0.);
        if canvas.width < 8. || canvas.height < 8. { return; }

        let cam = Camera::from_orbit(
            self.orbit_target, self.orbit_yaw, self.orbit_pitch, self.orbit_dist, canvas,
        );
        self.process_input(ui, canvas, &cam, scene, selected);

        if !self.framebuffer.is_empty() && self.framebuffer_width > 0 && self.framebuffer_height > 0 {
            let ia = self.framebuffer_width as f64 / self.framebuffer_height as f64;
            let ca = canvas.width / canvas.height;
            let (iw,ih) = if ia > ca { (canvas.width, canvas.width/ia) } else { (canvas.height*ia, canvas.height) };
            ui.draw_list.image(
                Rect::new(canvas.x+(canvas.width-iw)*0.5, canvas.y+(canvas.height-ih)*0.5, iw, ih),
                self.framebuffer.clone(), self.framebuffer_width, self.framebuffer_height, [1.;4],
            );
        }

        ui.draw_list.push_clip(canvas);
        if self.show_grid { self.draw_grid_3d(ui, &cam); }
        self.draw_objects_3d(ui, &cam, scene, selected);
        if let Some(sel_id) = selected {
            if let Some(obj) = scene.get(sel_id) {
                self.draw_gizmo(ui, &cam, obj.position, self.gizmo_mode);
            }
        }
        ui.draw_list.pop_clip();

        if self.show_stats_overlay {
            let _ = WidgetId::NONE;
            let ov = Rect::new(canvas.x+8., canvas.y+8., 252., 82.);
            ui.draw_list.rect(ov, [0.,0.,0.,0.55], 4.);
            ui.draw_list.text(Vec2::new(ov.x+8.,ov.y+8.),
                format!("{}x{}  frame {}", canvas.width as u32, canvas.height as u32, ui.frame_index),
                ui.theme.palette.text, ui.theme.metrics.font_size_small);
            ui.draw_list.text(Vec2::new(ov.x+8.,ov.y+26.),
                format!("yaw:{:.0}  pitch:{:.0}  dist:{:.1}", self.orbit_yaw, self.orbit_pitch, self.orbit_dist),
                ui.theme.palette.text_muted, ui.theme.metrics.font_size_small);
            ui.draw_list.text(Vec2::new(ov.x+8.,ov.y+44.),
                "LMB orbit  MMB pan  Scroll zoom  Click select",
                ui.theme.palette.text_muted, ui.theme.metrics.font_size_small);
            ui.draw_list.text(Vec2::new(ov.x+8.,ov.y+62.),
                "G/R/S gizmo  F frame  Del delete  Ctrl+D dup",
                ui.theme.palette.text_muted, ui.theme.metrics.font_size_small);
        }
    }

    fn process_input(
        &mut self, ui: &mut UiContext, canvas: Rect, cam: &Camera,
        scene: &Scene, selected: Option<ObjectId>,
    ) {
        let mx = ui.input.pointer.x; let my = ui.input.pointer.y;
        let in_canvas = canvas.contains(Vec2::new(mx, my));
        let (dx,dy) = ui.input.pointer.delta();
        let left   = ui.input.pointer.left_down;
        let middle = ui.input.pointer.middle_down;
        let right  = ui.input.pointer.right_down;
        let scroll = ui.input.pointer.scroll_y;

        let just_left          = left   && !self.was_left_down;
        let just_released_left = !left  && self.was_left_down;
        let just_middle        = middle && !self.was_middle_down;

        if in_canvas && scroll.abs() > 1e-6 {
            let f = if scroll > 0. { 0.85 } else { 1.15 };
            self.orbit_dist = (self.orbit_dist*f).clamp(0.1, 2000.);
        }

        if just_left && in_canvas {
            self.left_press_pos = Some((mx,my));
            self.left_drag_dist = 0.;
            let gizmo_axis = selected
                .and_then(|id| scene.get(id))
                .and_then(|obj| self.hit_gizmo(cam, obj.position, mx, my));
            if let Some(axis) = gizmo_axis {
                if let Some(obj) = selected.and_then(|id| scene.get(id)) {
                    let aw: [f64;3] = match axis { 0=>[1.,0.,0.], 1=>[0.,1.,0.], _=>[0.,0.,1.] };
                    let dir = match (cam.project(obj.position), cam.project(vadd(obj.position,aw))) {
                        (Some(sp),Some(st)) => {
                            let ddx=st.x-sp.x; let ddy=st.y-sp.y;
                            let d=(ddx*ddx+ddy*ddy).sqrt();
                            if d>0.5 { (ddx/d,ddy/d) } else { (1.,0.) }
                        }
                        _ => (1.,0.),
                    };
                    self.gizmo_drag = Some(GizmoDrag {
                        axis, start_mouse:(mx,my), start_pos:obj.position, axis_screen_dir:dir,
                    });
                    self.orbit_active = false;
                }
            } else {
                self.orbit_active = true;
            }
        }

        if just_middle && in_canvas { self.pan_active = true; }
        if !middle { self.pan_active = false; }

        if left {
            if let Some(ref drag) = self.gizmo_drag {
                if let Some(sel_id) = selected {
                    let sd = (mx-drag.start_mouse.0)*drag.axis_screen_dir.0
                           + (my-drag.start_mouse.1)*drag.axis_screen_dir.1;
                    let wd = sd/cam.fov_scale*self.orbit_dist.max(0.5);
                    let mut np = drag.start_pos;
                    np[drag.axis as usize] += wd;
                    self.pending_move = Some((sel_id, np));
                }
            }
        }

        if left && self.orbit_active {
            if let Some(pp) = self.left_press_pos {
                self.left_drag_dist = ((mx-pp.0).powi(2)+(my-pp.1).powi(2)).sqrt();
            }
            if self.left_drag_dist > 4. {
                self.orbit_yaw   = (self.orbit_yaw + dx*0.5) % 360.;
                self.orbit_pitch = (self.orbit_pitch - dy*0.5).clamp(-89., 89.);
            }
        }

        if right {
            self.orbit_yaw   = (self.orbit_yaw + dx*0.5) % 360.;
            self.orbit_pitch = (self.orbit_pitch - dy*0.5).clamp(-89., 89.);
        }

        if middle && self.pan_active {
            let sp = self.orbit_dist*0.003;
            self.orbit_target[0] -= (cam.right[0]*dx - cam.up[0]*dy)*sp;
            self.orbit_target[1] -= (cam.right[1]*dx - cam.up[1]*dy)*sp;
            self.orbit_target[2] -= (cam.right[2]*dx - cam.up[2]*dy)*sp;
        }

        if just_released_left {
            if let Some(drag) = self.gizmo_drag.take() {
                if let Some(sel_id) = selected {
                    if let Some((_,fp)) = self.pending_move {
                        self.transform_commit = Some((sel_id, drag.start_pos, fp));
                    }
                }
            } else if self.orbit_active && self.left_drag_dist < 4. {
                if let Some(pp) = self.left_press_pos {
                    if canvas.contains(Vec2::new(pp.0,pp.1)) {
                        self.pick(cam, scene, pp.0, pp.1);
                    }
                }
            }
            self.orbit_active   = false;
            self.left_press_pos = None;
            self.left_drag_dist = 0.;
        }

        self.was_left_down   = left;
        self.was_middle_down = middle;
        self.was_right_down  = right;
    }

    fn hit_gizmo(&self, cam: &Camera, pos: [f64;3], mx: f64, my: f64) -> Option<u8> {
        for (i,axis) in [[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]].iter().enumerate() {
            if let Some(st) = cam.project(vadd(pos, vscale(*axis, 1.2))) {
                if ((st.x-mx).powi(2)+(st.y-my).powi(2)).sqrt() < 14. {
                    return Some(i as u8);
                }
            }
        }
        None
    }

    fn pick(&mut self, cam: &Camera, scene: &Scene, mx: f64, my: f64) {
        let mut best = 28.0f64;
        let mut best_id = None;
        for obj in &scene.objects {
            if !obj.visible { continue; }
            if let Some(sc) = cam.project(obj.position) {
                let d = ((sc.x-mx).powi(2)+(sc.y-my).powi(2)).sqrt();
                if d < best { best = d; best_id = Some(obj.id); }
            }
        }
        self.pending_select = Some(best_id);
    }

    fn draw_grid_3d(&self, ui: &mut UiContext, cam: &Camera) {
        let size = 12i32;
        let col  = [ui.theme.palette.border[0],ui.theme.palette.border[1],ui.theme.palette.border[2],0.30];
        let cola = [ui.theme.palette.border[0],ui.theme.palette.border[1],ui.theme.palette.border[2],0.72];
        for i in -size..=size {
            let v = i as f64;
            let (c,w) = if i==0 { (cola,1.5) } else { (col,0.5) };
            if let (Some(a),Some(b)) = (cam.project([v,0.,-size as f64]), cam.project([v,0.,size as f64])) {
                ui.draw_list.line(a,b,c,w);
            }
            if let (Some(a),Some(b)) = (cam.project([-size as f64,0.,v]), cam.project([size as f64,0.,v])) {
                ui.draw_list.line(a,b,c,w);
            }
        }
        if let (Some(a),Some(b)) = (cam.project([0.,-4.,0.]), cam.project([0.,4.,0.])) {
            ui.draw_list.line(a, b, [0.3,0.85,0.3,0.75], 1.5);
        }
    }

    fn draw_objects_3d(&self, ui: &mut UiContext, cam: &Camera, scene: &Scene, selected: Option<ObjectId>) {
        for obj in &scene.objects {
            if !obj.visible { continue; }
            let is_sel = selected == Some(obj.id);
            let color  = object_color(&obj.kind, is_sel);
            let thick  = if is_sel { 2. } else { 1. };
            for (a,b) in &object_edges(&obj.kind) {
                let wa = vadd(obj.position, rotate_euler(
                    [a[0]*obj.scale[0],a[1]*obj.scale[1],a[2]*obj.scale[2]],
                    obj.rotation[0],obj.rotation[1],obj.rotation[2]));
                let wb = vadd(obj.position, rotate_euler(
                    [b[0]*obj.scale[0],b[1]*obj.scale[1],b[2]*obj.scale[2]],
                    obj.rotation[0],obj.rotation[1],obj.rotation[2]));
                if let (Some(sa),Some(sb)) = (cam.project(wa), cam.project(wb)) {
                    ui.draw_list.line(sa, sb, color, thick);
                }
            }
            if let Some(sc) = cam.project(obj.position) {
                ui.draw_list.text(
                    Vec2::new(sc.x+8., sc.y-10.), &obj.name,
                    if is_sel { ui.theme.palette.text } else { ui.theme.palette.text_muted },
                    ui.theme.metrics.font_size_small,
                );
            }
        }
    }

    fn draw_gizmo(&self, ui: &mut UiContext, cam: &Camera, pos: [f64;3], mode: u32) {
        let axes: [([f64;3],[f64;4]);3] = [
            ([1.,0.,0.],[1.0,0.25,0.25,1.0]),
            ([0.,1.,0.],[0.25,1.0,0.25,1.0]),
            ([0.,0.,1.],[0.25,0.55,1.0,1.0]),
        ];
        if mode == 0 {
            for (ax,col) in &axes {
                if let (Some(sp),Some(st)) = (cam.project(pos), cam.project(vadd(pos,vscale(*ax,1.2)))) {
                    ui.draw_list.line(sp, st, *col, 2.5);
                    let ddx=st.x-sp.x; let ddy=st.y-sp.y;
                    let d=(ddx*ddx+ddy*ddy).sqrt().max(0.1);
                    let (nx,ny) = (ddx/d, ddy/d);
                    ui.draw_list.line(st, Vec2::new(st.x-nx*9.-ny*4.5, st.y-ny*9.+nx*4.5), *col, 2.0);
                    ui.draw_list.line(st, Vec2::new(st.x-nx*9.+ny*4.5, st.y-ny*9.-nx*4.5), *col, 2.0);
                }
            }
        } else if mode == 1 {
            let segs = 32;
            for (ax,col) in &axes {
                for i in 0..segs {
                    let a0 = i as f64/segs as f64*std::f64::consts::TAU;
                    let a1 = (i+1) as f64/segs as f64*std::f64::consts::TAU;
                    if let (Some(s0),Some(s1)) = (
                        cam.project(circle_on_plane(pos,*ax,0.85,a0)),
                        cam.project(circle_on_plane(pos,*ax,0.85,a1)),
                    ) { ui.draw_list.line(s0, s1, *col, 2.0); }
                }
            }
        } else if mode == 2 {
            for (ax,col) in &axes {
                if let (Some(sp),Some(st)) = (cam.project(pos), cam.project(vadd(pos,vscale(*ax,1.0)))) {
                    ui.draw_list.line(sp, st, *col, 2.0);
                    let c = 5.0;
                    ui.draw_list.rect(Rect::new(st.x-c, st.y-c, c*2., c*2.), *col, 2.0);
                }
            }
        }
    }
}
