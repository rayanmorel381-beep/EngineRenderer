/// Simple affine transform description (position + scale + rotation Euler).
/// No matrix math here — just a data struct the renderer can consume.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: [f64; 3],
    pub scale: [f64; 3],
    /// Euler angles in degrees (yaw, pitch, roll).
    pub rotation_degrees: [f64; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            scale: [1.0; 3],
            rotation_degrees: [0.0; 3],
        }
    }
}

impl Transform {
    pub fn at(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: [x, y, z],
            ..Default::default()
        }
    }

    pub fn with_scale(mut self, sx: f64, sy: f64, sz: f64) -> Self {
        self.scale = [sx, sy, sz];
        self
    }

    pub fn uniform_scale(mut self, s: f64) -> Self {
        self.scale = [s, s, s];
        self
    }

    pub fn with_rotation(mut self, yaw: f64, pitch: f64, roll: f64) -> Self {
        self.rotation_degrees = [yaw, pitch, roll];
        self
    }
}
