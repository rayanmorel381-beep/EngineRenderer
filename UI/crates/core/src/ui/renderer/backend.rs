use crate::ui::immediate::draw_list::DrawList;

pub trait RendererBackend {
    fn init(&mut self, viewport_w: u32, viewport_h: u32);
    fn resize(&mut self, viewport_w: u32, viewport_h: u32);
    fn begin_frame(&mut self, clear_color: [f64; 4]);
    fn submit(&mut self, draw_list: &DrawList);
    fn end_frame(&mut self);
}
