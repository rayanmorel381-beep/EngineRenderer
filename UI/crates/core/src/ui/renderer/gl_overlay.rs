use core::ffi::c_void;

use enginerenderer::api::display::NativeWindow;

use crate::ui::immediate::draw_list::{DrawCommand, DrawList};
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::renderer::backend::RendererBackend;
use crate::ui::renderer::gl_bindings::{
    GL_ARRAY_BUFFER, GL_BLEND, GL_CLAMP_TO_EDGE, GL_COLOR_BUFFER_BIT, GL_COMPILE_STATUS,
    GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_FRAGMENT_SHADER, GL_LINES, GL_LINEAR, GL_LINK_STATUS,
    GL_ONE_MINUS_SRC_ALPHA, GL_R8, GL_RED, GL_RGBA, GL_RGBA8, GL_SCISSOR_TEST,
    GL_SRC_ALPHA, GL_TEXTURE0, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER,
    GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T, GL_TRIANGLES, GL_UNPACK_ALIGNMENT, GL_UNSIGNED_BYTE,
    GL_VERTEX_SHADER, GLchar, GLfloat, GLint, GLsizei, GLuint, GlFns,
};
use crate::ui::renderer::shaders::{FRAGMENT_SHADER, VERTEX_SHADER};
use crate::ui::renderer::vertex::{Vertex, VertexBatch};
use crate::ui::text::font::{self, Font, RASTER_SIZE};
use crate::ui::text::glyph_atlas::GlyphAtlas;

const VERTEX_STRIDE: GLsizei = core::mem::size_of::<Vertex>() as GLsizei;
const ATTR_POS_OFFSET: usize = 0;
const ATTR_UV_OFFSET: usize = 8;
const ATTR_COLOR_OFFSET: usize = 16;
const ATTR_MODE_OFFSET: usize = 32;
const ATTR_LOCAL_OFFSET: usize = 36;
const ATTR_PARAMS_OFFSET: usize = 44;

#[derive(Copy, Clone, Debug)]
struct Segment {
    start: u32,
    count: u32,
    primitive: u32,
    texture: GLuint,
}

pub struct GlOverlay {
    pub initialized: bool,
    pub viewport_w: u32,
    pub viewport_h: u32,
    pub atlas: Option<GlyphAtlas>,
    pub clip_stack: Vec<Rect>,
    pub batch: VertexBatch,
    fns: GlFns,
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    atlas_texture: GLuint,
    u_viewport: GLint,
    u_tex: GLint,
    vbo_capacity: GLsizei,
    segments: Vec<Segment>,
    transient_textures: Vec<GLuint>,
    open_segment_start: u32,
}

impl Default for GlOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl GlOverlay {
    pub fn new() -> Self {
        Self {
            initialized: false,
            viewport_w: 0,
            viewport_h: 0,
            atlas: None,
            clip_stack: Vec::new(),
            batch: VertexBatch::new(),
            fns: GlFns::default(),
            program: 0,
            vao: 0,
            vbo: 0,
            atlas_texture: 0,
            u_viewport: -1,
            u_tex: -1,
            vbo_capacity: 0,
            segments: Vec::new(),
            transient_textures: Vec::new(),
            open_segment_start: 0,
        }
    }

    pub fn attach(&mut self, window: &NativeWindow) {
        window.make_current();
        self.fns = GlFns::load(window);
        if !self.fns.complete() {
            return;
        }
        let atlas = GlyphAtlas::build_default();
        self.create_program();
        self.create_buffers();
        self.upload_atlas(&atlas);
        self.atlas = Some(atlas);
        self.initialized = true;
    }

    fn create_program(&mut self) {
        let create_shader = self.fns.create_shader.unwrap();
        let compile = self.fns.compile_shader.unwrap();
        let create_program = self.fns.create_program.unwrap();
        let attach = self.fns.attach_shader.unwrap();
        let link = self.fns.link_program.unwrap();
        let get_uniform = self.fns.get_uniform_location.unwrap();
        let get_shader_iv = self.fns.get_shader_iv;
        let get_program_iv = self.fns.get_program_iv;

        let vs = create_shader(GL_VERTEX_SHADER);
        let fs = create_shader(GL_FRAGMENT_SHADER);
        upload_shader(&self.fns, vs, VERTEX_SHADER);
        compile(vs);
        check_shader(get_shader_iv, vs);
        upload_shader(&self.fns, fs, FRAGMENT_SHADER);
        compile(fs);
        check_shader(get_shader_iv, fs);

        let program = create_program();
        attach(program, vs);
        attach(program, fs);
        link(program);
        if let Some(getp) = get_program_iv {
            let mut status: GLint = 0;
            getp(program, GL_LINK_STATUS, &mut status);
        }
        self.program = program;
        self.u_viewport = get_uniform(program, c"u_viewport".as_ptr() as *const GLchar);
        self.u_tex = get_uniform(program, c"u_tex".as_ptr() as *const GLchar);
    }

    fn create_buffers(&mut self) {
        let gen_vao = self.fns.gen_vertex_arrays.unwrap();
        let bind_vao = self.fns.bind_vertex_array.unwrap();
        let gen_buf = self.fns.gen_buffers.unwrap();
        let bind_buf = self.fns.bind_buffer.unwrap();
        let buffer_data = self.fns.buffer_data.unwrap();
        let enable_attr = self.fns.enable_vertex_attrib_array.unwrap();
        let attr_ptr = self.fns.vertex_attrib_pointer.unwrap();

        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        gen_vao(1, &mut vao);
        bind_vao(vao);
        gen_buf(1, &mut vbo);
        bind_buf(GL_ARRAY_BUFFER, vbo);
        let initial_capacity = 64 * 1024;
        buffer_data(
            GL_ARRAY_BUFFER,
            (initial_capacity as usize * core::mem::size_of::<Vertex>()) as isize,
            core::ptr::null(),
            GL_DYNAMIC_DRAW,
        );
        self.vbo_capacity = initial_capacity;

        enable_attr(0);
        attr_ptr(0, 2, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_POS_OFFSET as *const c_void);
        enable_attr(1);
        attr_ptr(1, 2, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_UV_OFFSET as *const c_void);
        enable_attr(2);
        attr_ptr(2, 4, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_COLOR_OFFSET as *const c_void);
        enable_attr(3);
        attr_ptr(3, 1, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_MODE_OFFSET as *const c_void);
        enable_attr(4);
        attr_ptr(4, 2, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_LOCAL_OFFSET as *const c_void);
        enable_attr(5);
        attr_ptr(5, 4, GL_FLOAT, GL_FALSE, VERTEX_STRIDE, ATTR_PARAMS_OFFSET as *const c_void);

        self.vao = vao;
        self.vbo = vbo;
    }

    fn upload_atlas(&mut self, atlas: &GlyphAtlas) {
        let gen_tex = self.fns.gen_textures.unwrap();
        let bind_tex = self.fns.bind_texture.unwrap();
        let tex_image = self.fns.tex_image_2d.unwrap();
        let tex_param = self.fns.tex_parameter_i.unwrap();
        let pixel_storei = self.fns.pixel_storei.unwrap();

        let mut tex: GLuint = 0;
        gen_tex(1, &mut tex);
        bind_tex(GL_TEXTURE_2D, tex);
        pixel_storei(GL_UNPACK_ALIGNMENT, 1);
        tex_image(
            GL_TEXTURE_2D,
            0,
            GL_R8,
            atlas.texture_width as GLsizei,
            atlas.texture_height as GLsizei,
            0,
            GL_RED,
            GL_UNSIGNED_BYTE,
            atlas.pixels.as_ptr() as *const c_void,
        );
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        self.atlas_texture = tex;
    }

    fn reupload_atlas(&mut self) {
        if let Some(atlas) = self.atlas.as_mut()
            && atlas.dirty
            && self.atlas_texture != 0
        {
            let bind_tex = self.fns.bind_texture.unwrap();
            let tex_image = self.fns.tex_image_2d.unwrap();
            let pixel_storei = self.fns.pixel_storei.unwrap();
            bind_tex(GL_TEXTURE_2D, self.atlas_texture);
            pixel_storei(GL_UNPACK_ALIGNMENT, 1);
            tex_image(
                GL_TEXTURE_2D,
                0,
                GL_R8,
                atlas.texture_width as GLsizei,
                atlas.texture_height as GLsizei,
                0,
                GL_RED,
                GL_UNSIGNED_BYTE,
                atlas.pixels.as_ptr() as *const c_void,
            );
            atlas.dirty = false;
        }
    }

    fn upload_user_image(&mut self, pixels: &[u8], width: u32, height: u32) -> GLuint {
        let gen_tex = self.fns.gen_textures.unwrap();
        let bind_tex = self.fns.bind_texture.unwrap();
        let tex_image = self.fns.tex_image_2d.unwrap();
        let tex_param = self.fns.tex_parameter_i.unwrap();
        let pixel_storei = self.fns.pixel_storei.unwrap();

        let mut tex: GLuint = 0;
        gen_tex(1, &mut tex);
        bind_tex(GL_TEXTURE_2D, tex);
        pixel_storei(GL_UNPACK_ALIGNMENT, 1);
        tex_image(
            GL_TEXTURE_2D,
            0,
            GL_RGBA8,
            width as GLsizei,
            height as GLsizei,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            pixels.as_ptr() as *const c_void,
        );
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        tex_param(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        tex
    }

    fn record_text(
        &mut self,
        position: Vec2,
        text: &str,
        color: [f64; 4],
        font_size: f64,
    ) {
        let Some(atlas) = self.atlas.as_mut() else {
            return;
        };
        let scale = font_size / RASTER_SIZE;
        let baseline_y = position.y + Font::REGULAR.ascent(font_size);
        let tex_w = atlas.texture_width as f32;
        let tex_h = atlas.texture_height as f32;
        let mut pen_x = position.x;
        let mut prev: Option<(char, &'static fontdue::Font)> = None;
        for ch in text.chars() {
            let codepoint = ch as u32;
            let font_ref = font::font_for_codepoint(codepoint);
            if let Some((p, prev_font)) = prev
                && std::ptr::eq(prev_font, font_ref)
            {
                pen_x += font_ref
                    .horizontal_kern(p, ch, font_size as f32)
                    .unwrap_or(0.0) as f64;
            }
            let glyph = match atlas.ensure(codepoint) {
                Some(g) => g,
                None => {
                    prev = Some((ch, font_ref));
                    continue;
                }
            };
            let advance_at_size = font_ref.metrics(ch, font_size as f32).advance_width as f64;
            if glyph.width > 0 && glyph.height > 0 {
                let glyph_w = glyph.width as f64 * scale;
                let glyph_h = glyph.height as f64 * scale;
                let glyph_x = pen_x + glyph.xmin * scale;
                let glyph_y = baseline_y - (glyph.ymin + glyph.height as f64) * scale;
                let dst = Rect::new(glyph_x, glyph_y, glyph_w, glyph_h);
                let u0 = glyph.atlas_x as f32 / tex_w;
                let v0 = glyph.atlas_y as f32 / tex_h;
                let u1 = (glyph.atlas_x + glyph.width) as f32 / tex_w;
                let v1 = (glyph.atlas_y + glyph.height) as f32 / tex_h;
                self.batch
                    .push_textured_quad(dst, [(u0, v0), (u1, v0), (u1, v1), (u0, v1)], color);
            }
            pen_x += advance_at_size;
            prev = Some((ch, font_ref));
        }
    }

    fn close_atlas_segment(&mut self) {
        let end = self.batch.triangles.len() as u32;
        if end > self.open_segment_start {
            self.segments.push(Segment {
                start: self.open_segment_start,
                count: end - self.open_segment_start,
                primitive: GL_TRIANGLES,
                texture: self.atlas_texture,
            });
            self.open_segment_start = end;
        }
    }

    fn handle_image(&mut self, rect: Rect, pixels: &[u8], width: u32, height: u32, tint: [f64; 4]) {
        if !self.initialized || width == 0 || height == 0 {
            return;
        }
        self.close_atlas_segment();
        let texture = self.upload_user_image(pixels, width, height);
        self.transient_textures.push(texture);
        let start = self.batch.triangles.len() as u32;
        self.batch.push_image_quad(rect, tint);
        let end = self.batch.triangles.len() as u32;
        self.segments.push(Segment {
            start,
            count: end - start,
            primitive: GL_TRIANGLES,
            texture,
        });
        self.open_segment_start = end;
    }

    fn flush(&mut self) {
        if !self.initialized {
            return;
        }
        self.reupload_atlas();
        let bind_vao = self.fns.bind_vertex_array.unwrap();
        let bind_buf = self.fns.bind_buffer.unwrap();
        let buffer_data = self.fns.buffer_data.unwrap();
        let buffer_sub_data = self.fns.buffer_sub_data.unwrap();
        let use_program = self.fns.use_program.unwrap();
        let uniform2f = self.fns.uniform2f.unwrap();
        let uniform1i = self.fns.uniform1i.unwrap();
        let active_texture = self.fns.active_texture.unwrap();
        let bind_texture = self.fns.bind_texture.unwrap();
        let draw_arrays = self.fns.draw_arrays.unwrap();

        use_program(self.program);
        uniform2f(self.u_viewport, self.viewport_w as GLfloat, self.viewport_h as GLfloat);
        active_texture(GL_TEXTURE0);
        uniform1i(self.u_tex, 0);

        bind_vao(self.vao);
        bind_buf(GL_ARRAY_BUFFER, self.vbo);

        if !self.batch.triangles.is_empty() {
            let bytes = (self.batch.triangles.len() * core::mem::size_of::<Vertex>()) as isize;
            if self.batch.triangles.len() as GLsizei > self.vbo_capacity {
                buffer_data(
                    GL_ARRAY_BUFFER,
                    bytes,
                    self.batch.triangles.as_ptr() as *const c_void,
                    GL_DYNAMIC_DRAW,
                );
                self.vbo_capacity = self.batch.triangles.len() as GLsizei;
            } else {
                buffer_sub_data(
                    GL_ARRAY_BUFFER,
                    0,
                    bytes,
                    self.batch.triangles.as_ptr() as *const c_void,
                );
            }
            for seg in &self.segments {
                bind_texture(GL_TEXTURE_2D, seg.texture);
                draw_arrays(seg.primitive, seg.start as GLint, seg.count as GLsizei);
            }
        }

        if !self.batch.lines.is_empty() {
            let bytes = (self.batch.lines.len() * core::mem::size_of::<Vertex>()) as isize;
            if self.batch.lines.len() as GLsizei > self.vbo_capacity {
                buffer_data(
                    GL_ARRAY_BUFFER,
                    bytes,
                    self.batch.lines.as_ptr() as *const c_void,
                    GL_DYNAMIC_DRAW,
                );
                self.vbo_capacity = self.batch.lines.len() as GLsizei;
            } else {
                buffer_sub_data(
                    GL_ARRAY_BUFFER,
                    0,
                    bytes,
                    self.batch.lines.as_ptr() as *const c_void,
                );
            }
            bind_texture(GL_TEXTURE_2D, self.atlas_texture);
            draw_arrays(GL_LINES, 0, self.batch.lines.len() as GLsizei);
        }
    }

    fn release_transient_textures(&mut self) {
        if self.transient_textures.is_empty() {
            return;
        }
        if let Some(delete) = self.fns.delete_textures {
            delete(
                self.transient_textures.len() as GLsizei,
                self.transient_textures.as_ptr(),
            );
        }
        self.transient_textures.clear();
    }
}

fn upload_shader(fns: &GlFns, shader: GLuint, source: &str) {
    let shader_source = fns.shader_source.unwrap();
    let mut owned = source.as_bytes().to_vec();
    owned.push(0);
    let ptr = owned.as_ptr() as *const GLchar;
    let len = (owned.len() - 1) as GLint;
    shader_source(shader, 1, &ptr, &len);
}

fn check_shader(get_shader_iv: Option<extern "system" fn(GLuint, u32, *mut GLint)>, shader: GLuint) {
    if let Some(f) = get_shader_iv {
        let mut status: GLint = 0;
        f(shader, GL_COMPILE_STATUS, &mut status);
    }
}

impl RendererBackend for GlOverlay {
    fn init(&mut self, viewport_w: u32, viewport_h: u32) {
        self.viewport_w = viewport_w;
        self.viewport_h = viewport_h;
    }

    fn resize(&mut self, viewport_w: u32, viewport_h: u32) {
        self.viewport_w = viewport_w;
        self.viewport_h = viewport_h;
    }

    fn begin_frame(&mut self, clear_color: [f64; 4]) {
        self.batch.clear();
        self.clip_stack.clear();
        self.segments.clear();
        self.open_segment_start = 0;
        if !self.initialized {
            return;
        }
        if let (
            Some(viewport),
            Some(clear_c),
            Some(clear),
            Some(enable),
            Some(disable),
            Some(blend),
        ) = (
            self.fns.viewport,
            self.fns.clear_color,
            self.fns.clear,
            self.fns.enable,
            self.fns.disable,
            self.fns.blend_func,
        ) {
            viewport(0, 0, self.viewport_w as GLsizei, self.viewport_h as GLsizei);
            clear_c(
                clear_color[0] as GLfloat,
                clear_color[1] as GLfloat,
                clear_color[2] as GLfloat,
                clear_color[3] as GLfloat,
            );
            clear(GL_COLOR_BUFFER_BIT);
            enable(GL_BLEND);
            blend(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
            disable(GL_SCISSOR_TEST);
        }
    }

    fn submit(&mut self, draw_list: &DrawList) {
        for cmd in &draw_list.commands {
            match cmd {
                DrawCommand::Rect {
                    rect,
                    color,
                    corner_radius,
                } => {
                    if *corner_radius > 0.5 {
                        self.batch.push_round_rect(*rect, *color, *corner_radius);
                    } else {
                        self.batch.push_rect(*rect, *color);
                    }
                }
                DrawCommand::RectOutline {
                    rect,
                    color,
                    thickness,
                    corner_radius,
                } => {
                    if *corner_radius > 0.5 {
                        self.batch.push_round_rect_outline(
                            *rect,
                            *color,
                            *corner_radius,
                            *thickness,
                        );
                    } else {
                        self.batch.push_rect_outline(*rect, *color, *thickness);
                    }
                }
                DrawCommand::Line {
                    from,
                    to,
                    color,
                    ..
                } => self.batch.push_line(*from, *to, *color),
                DrawCommand::Text {
                    position,
                    text,
                    color,
                    font_size,
                } => self.record_text(*position, text, *color, *font_size),
                DrawCommand::Image {
                    rect,
                    pixels,
                    width,
                    height,
                    tint,
                } => self.handle_image(*rect, pixels, *width, *height, *tint),
                DrawCommand::Shadow {
                    rect,
                    color,
                    corner_radius,
                    spread,
                } => self
                    .batch
                    .push_drop_shadow(*rect, *color, *corner_radius, *spread),
                DrawCommand::Clip { rect } => self.clip_stack.push(*rect),
                DrawCommand::PopClip => {
                    self.clip_stack.pop();
                }
            }
        }
        self.close_atlas_segment();
    }

    fn end_frame(&mut self) {
        self.flush();
        self.release_transient_textures();
    }
}
