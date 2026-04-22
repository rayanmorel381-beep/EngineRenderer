/// Object transform expressed as position, scale, and Euler rotation.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// World-space position.
    pub position: [f64; 3],
    /// Non-uniform scale factors.
    pub scale: [f64; 3],
    /// Rotation angles in degrees: yaw, pitch, roll.
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
    /// Creates a transform at the provided position.
    pub fn at(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: [x, y, z],
            ..Default::default()
        }
    }

    /// Sets non-uniform scale values.
    pub fn with_scale(mut self, sx: f64, sy: f64, sz: f64) -> Self {
        self.scale = [sx, sy, sz];
        self
    }

    /// Sets a uniform scale on all axes.
    pub fn uniform_scale(mut self, s: f64) -> Self {
        self.scale = [s, s, s];
        self
    }

    /// Sets rotation in degrees as yaw, pitch, and roll.
    pub fn with_rotation(mut self, yaw: f64, pitch: f64, roll: f64) -> Self {
        self.rotation_degrees = [yaw, pitch, roll];
        self
    }
}
