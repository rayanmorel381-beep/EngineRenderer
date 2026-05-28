use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::palette::Rgba;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub mode: f32,
    pub local_x: f32,
    pub local_y: f32,
    pub half_w: f32,
    pub half_h: f32,
    pub radius: f32,
    pub softness: f32,
}

pub const MODE_SOLID: f32 = 0.0;
pub const MODE_TEXTURED: f32 = 1.0;
pub const MODE_IMAGE: f32 = 2.0;
pub const MODE_ROUND_FILL: f32 = 3.0;
pub const MODE_ROUND_OUTLINE: f32 = 4.0;
pub const MODE_DROP_SHADOW: f32 = 5.0;
pub const MODE_ROUND_GRADIENT: f32 = 6.0;

#[derive(Copy, Clone, Debug)]
pub struct SdfParams {
    pub local: (f32, f32),
    pub mode: f32,
    pub half_w: f32,
    pub half_h: f32,
    pub radius: f32,
    pub softness: f32,
}

impl Vertex {
    pub fn solid(p: Vec2, color: Rgba) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
            u: 0.0,
            v: 0.0,
            r: color[0] as f32,
            g: color[1] as f32,
            b: color[2] as f32,
            a: color[3] as f32,
            mode: MODE_SOLID,
            local_x: 0.0,
            local_y: 0.0,
            half_w: 0.0,
            half_h: 0.0,
            radius: 0.0,
            softness: 0.0,
        }
    }

    pub fn textured(p: Vec2, uv: (f32, f32), color: Rgba) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
            u: uv.0,
            v: uv.1,
            r: color[0] as f32,
            g: color[1] as f32,
            b: color[2] as f32,
            a: color[3] as f32,
            mode: MODE_TEXTURED,
            local_x: 0.0,
            local_y: 0.0,
            half_w: 0.0,
            half_h: 0.0,
            radius: 0.0,
            softness: 0.0,
        }
    }

    pub fn image(p: Vec2, uv: (f32, f32), tint: Rgba) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
            u: uv.0,
            v: uv.1,
            r: tint[0] as f32,
            g: tint[1] as f32,
            b: tint[2] as f32,
            a: tint[3] as f32,
            mode: MODE_IMAGE,
            local_x: 0.0,
            local_y: 0.0,
            half_w: 0.0,
            half_h: 0.0,
            radius: 0.0,
            softness: 0.0,
        }
    }

    pub fn sdf(p: Vec2, color: Rgba, params: SdfParams) -> Self {
        Self {
            x: p.x as f32,
            y: p.y as f32,
            u: 0.0,
            v: 0.0,
            r: color[0] as f32,
            g: color[1] as f32,
            b: color[2] as f32,
            a: color[3] as f32,
            mode: params.mode,
            local_x: params.local.0,
            local_y: params.local.1,
            half_w: params.half_w,
            half_h: params.half_h,
            radius: params.radius,
            softness: params.softness,
        }
    }
}

pub struct VertexBatch {
    pub triangles: Vec<Vertex>,
    pub lines: Vec<Vertex>,
}

impl Default for VertexBatch {
    fn default() -> Self {
        Self {
            triangles: Vec::with_capacity(4096),
            lines: Vec::with_capacity(1024),
        }
    }
}

impl VertexBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.triangles.clear();
        self.lines.clear();
    }

    pub fn push_rect(&mut self, rect: Rect, color: Rgba) {
        let tl = Vec2::new(rect.x, rect.y);
        let tr = Vec2::new(rect.x + rect.width, rect.y);
        let bl = Vec2::new(rect.x, rect.y + rect.height);
        let br = Vec2::new(rect.x + rect.width, rect.y + rect.height);
        self.triangles.push(Vertex::solid(tl, color));
        self.triangles.push(Vertex::solid(tr, color));
        self.triangles.push(Vertex::solid(br, color));
        self.triangles.push(Vertex::solid(tl, color));
        self.triangles.push(Vertex::solid(br, color));
        self.triangles.push(Vertex::solid(bl, color));
    }

    pub fn push_rect_outline(&mut self, rect: Rect, color: Rgba, thickness: f64) {
        let t = thickness.max(1.0);
        self.push_rect(Rect::new(rect.x, rect.y, rect.width, t), color);
        self.push_rect(
            Rect::new(rect.x, rect.y + rect.height - t, rect.width, t),
            color,
        );
        self.push_rect(Rect::new(rect.x, rect.y, t, rect.height), color);
        self.push_rect(
            Rect::new(rect.x + rect.width - t, rect.y, t, rect.height),
            color,
        );
    }

    pub fn push_line(&mut self, from: Vec2, to: Vec2, color: Rgba) {
        self.lines.push(Vertex::solid(from, color));
        self.lines.push(Vertex::solid(to, color));
    }

    pub fn push_textured_quad(&mut self, rect: Rect, uv: [(f32, f32); 4], color: Rgba) {
        let tl = Vec2::new(rect.x, rect.y);
        let tr = Vec2::new(rect.x + rect.width, rect.y);
        let bl = Vec2::new(rect.x, rect.y + rect.height);
        let br = Vec2::new(rect.x + rect.width, rect.y + rect.height);
        self.triangles.push(Vertex::textured(tl, uv[0], color));
        self.triangles.push(Vertex::textured(tr, uv[1], color));
        self.triangles.push(Vertex::textured(br, uv[2], color));
        self.triangles.push(Vertex::textured(tl, uv[0], color));
        self.triangles.push(Vertex::textured(br, uv[2], color));
        self.triangles.push(Vertex::textured(bl, uv[3], color));
    }

    pub fn push_image_quad(&mut self, rect: Rect, tint: Rgba) {
        let tl = Vec2::new(rect.x, rect.y);
        let tr = Vec2::new(rect.x + rect.width, rect.y);
        let bl = Vec2::new(rect.x, rect.y + rect.height);
        let br = Vec2::new(rect.x + rect.width, rect.y + rect.height);
        self.triangles.push(Vertex::image(tl, (0.0, 0.0), tint));
        self.triangles.push(Vertex::image(tr, (1.0, 0.0), tint));
        self.triangles.push(Vertex::image(br, (1.0, 1.0), tint));
        self.triangles.push(Vertex::image(tl, (0.0, 0.0), tint));
        self.triangles.push(Vertex::image(br, (1.0, 1.0), tint));
        self.triangles.push(Vertex::image(bl, (0.0, 1.0), tint));
    }

    fn push_sdf_quad(
        &mut self,
        rect: Rect,
        color: Rgba,
        mode: f32,
        radius: f64,
        softness: f64,
        padding: f64,
    ) {
        let pad = padding.max(0.0);
        let x0 = rect.x - pad;
        let y0 = rect.y - pad;
        let x1 = rect.x + rect.width + pad;
        let y1 = rect.y + rect.height + pad;
        let half_w = (rect.width * 0.5) as f32;
        let half_h = (rect.height * 0.5) as f32;
        let cx = rect.x + rect.width * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let r = radius as f32;
        let s = softness as f32;
        let make = |p: Vec2, l: (f32, f32)| {
            Vertex::sdf(
                p,
                color,
                SdfParams {
                    local: l,
                    mode,
                    half_w,
                    half_h,
                    radius: r,
                    softness: s,
                },
            )
        };
        let v_tl = make(Vec2::new(x0, y0), ((x0 - cx) as f32, (y0 - cy) as f32));
        let v_tr = make(Vec2::new(x1, y0), ((x1 - cx) as f32, (y0 - cy) as f32));
        let v_bl = make(Vec2::new(x0, y1), ((x0 - cx) as f32, (y1 - cy) as f32));
        let v_br = make(Vec2::new(x1, y1), ((x1 - cx) as f32, (y1 - cy) as f32));
        self.triangles.push(v_tl);
        self.triangles.push(v_tr);
        self.triangles.push(v_br);
        self.triangles.push(v_tl);
        self.triangles.push(v_br);
        self.triangles.push(v_bl);
    }

    pub fn push_round_rect(&mut self, rect: Rect, color: Rgba, radius: f64) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        self.push_sdf_quad(rect, color, MODE_ROUND_FILL, radius, 0.75, 1.0);
    }

    pub fn push_round_rect_outline(
        &mut self,
        rect: Rect,
        color: Rgba,
        radius: f64,
        thickness: f64,
    ) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let half_thickness = thickness.max(1.0) * 0.5;
        self.push_sdf_quad(rect, color, MODE_ROUND_OUTLINE, radius, half_thickness, 2.0);
    }

    pub fn push_drop_shadow(&mut self, rect: Rect, color: Rgba, radius: f64, spread: f64) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let s = spread.max(2.0);
        self.push_sdf_quad(rect, color, MODE_DROP_SHADOW, radius, s, s);
    }

    pub fn push_round_rect_gradient(&mut self, rect: Rect, color: Rgba, radius: f64) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        self.push_sdf_quad(rect, color, MODE_ROUND_GRADIENT, radius, 0.75, 1.0);
    }
}
