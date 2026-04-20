use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            self
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn rgb(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<Vec3> for Vec4 {
    fn from(v: Vec3) -> Self {
        Vec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn as_flat_array(&self) -> &[f32; 16] {
        unsafe { std::mem::transmute(&self.m) }
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov / 2.0).tan();
        let nf = 1.0 / (near - far);

        let mut result = Mat4::IDENTITY;
        result.m[0][0] = f / aspect;
        result.m[1][1] = f;
        result.m[2][2] = (far + near) * nf;
        result.m[2][3] = -1.0;
        result.m[3][2] = 2.0 * far * near * nf;
        result.m[3][3] = 0.0;
        result
    }

    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
        let f = (center - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        let mut result = Mat4::IDENTITY;
        result.m[0][0] = s.x;
        result.m[0][1] = s.y;
        result.m[0][2] = s.z;
        result.m[1][0] = u.x;
        result.m[1][1] = u.y;
        result.m[1][2] = u.z;
        result.m[2][0] = -f.x;
        result.m[2][1] = -f.y;
        result.m[2][2] = -f.z;
        result.m[0][3] = -s.dot(eye);
        result.m[1][3] = -u.dot(eye);
        result.m[2][3] = f.dot(eye);
        result
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Mat4 {
        let mut result = Mat4::IDENTITY;
        result.m[3][0] = x;
        result.m[3][1] = y;
        result.m[3][2] = z;
        result
    }

    pub fn rotate(axis: Vec3, angle: f32) -> Mat4 {
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let axis = axis.normalize();

        let mut result = Mat4::IDENTITY;
        result.m[0][0] = t * axis.x * axis.x + c;
        result.m[0][1] = t * axis.x * axis.y + s * axis.z;
        result.m[0][2] = t * axis.x * axis.z - s * axis.y;

        result.m[1][0] = t * axis.x * axis.y - s * axis.z;
        result.m[1][1] = t * axis.y * axis.y + c;
        result.m[1][2] = t * axis.y * axis.z + s * axis.x;

        result.m[2][0] = t * axis.x * axis.z + s * axis.y;
        result.m[2][1] = t * axis.y * axis.z - s * axis.x;
        result.m[2][2] = t * axis.z * axis.z + c;
        result
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Mat4 {
        let mut result = Mat4::IDENTITY;
        result.m[0][0] = x;
        result.m[1][1] = y;
        result.m[2][2] = z;
        result
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, other: Mat4) -> Mat4 {
        let mut result = Mat4::IDENTITY;
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = 0.0;
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        result
    }
}
