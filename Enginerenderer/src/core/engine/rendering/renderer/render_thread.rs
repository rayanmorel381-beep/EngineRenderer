use crate::core::engine::rendering::framebuffer::buffer::FrameBuffer;
use std::sync::mpsc;
use std::thread;

pub enum RenderCommand {
    SubmitFrame(FrameBuffer),
    Resize(usize, usize),
    Shutdown,
}

pub struct RenderThread {
    sender: mpsc::SyncSender<RenderCommand>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RenderThread {
    pub fn spawn(channel_capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(channel_capacity);
        let handle = thread::spawn(move || render_loop(receiver));
        Self {
            sender,
            handle: Some(handle),
        }
    }

    pub fn submit_frame(&self, fb: FrameBuffer) {
        let _ = self.sender.send(RenderCommand::SubmitFrame(fb));
    }

    pub fn resize(&self, width: usize, height: usize) {
        let _ = self.sender.send(RenderCommand::Resize(width, height));
    }

    pub fn shutdown(mut self) {
        let _ = self.sender.send(RenderCommand::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        let _ = self.sender.send(RenderCommand::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn render_loop(receiver: mpsc::Receiver<RenderCommand>) {
    let mut current_width = 0usize;
    let mut current_height = 0usize;
    let mut composite_buffer: Vec<[f32; 4]> = Vec::new();
    composite_buffer.reserve(current_width.max(1) * current_height.max(1));
    loop {
        match receiver.recv() {
            Ok(RenderCommand::SubmitFrame(fb)) => {
                current_width = fb.width;
                current_height = fb.height;
                let required = current_width * current_height;
                if composite_buffer.len() != required {
                    composite_buffer.resize(required, [0.0, 0.0, 0.0, 1.0]);
                }
                for (dst, src) in composite_buffer.iter_mut().zip(fb.color.iter()) {
                    dst[0] = src.x as f32;
                    dst[1] = src.y as f32;
                    dst[2] = src.z as f32;
                    dst[3] = 1.0;
                }
            }
            Ok(RenderCommand::Resize(w, h)) => {
                current_width = w;
                current_height = h;
                composite_buffer.resize(current_width * current_height, [0.0, 0.0, 0.0, 1.0]);
            }
            Ok(RenderCommand::Shutdown) | Err(_) => break,
        }
    }
}
