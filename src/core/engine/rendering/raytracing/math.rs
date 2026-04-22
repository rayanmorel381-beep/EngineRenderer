use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

/// Three-dimensional vector used across rendering math.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl Vec3 {
    /// Zero vector constant.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    /// One vector constant.
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

    /// Creates a vector from components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector with the same value in each component.
    pub const fn splat(value: f64) -> Self {
        Self::new(value, value, value)
    }

    /// Dot product.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the component value by index.
    pub fn axis(self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    /// Cross product.
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Squared vector length.
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    /// Vector length.
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Returns a normalized vector.
    pub fn normalize(self) -> Self {
        let length = self.length();
        if length <= f64::EPSILON {
            Self::ZERO
        } else {
            self / length
        }
    }

    /// Reflects the vector around a normal.
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    /// Refracts the vector through a surface normal.
    pub fn refract(self, normal: Self, eta_ratio: f64) -> Self {
        let cos_theta = (-self).dot(normal).min(1.0);
        let perpendicular = (self + normal * cos_theta) * eta_ratio;
        let parallel = normal * -(1.0 - perpendicular.length_squared()).abs().sqrt();
        perpendicular + parallel
    }

    /// Clamps each component to the provided range.
    pub fn clamp(self, min_value: f64, max_value: f64) -> Self {
        Self::new(
            self.x.clamp(min_value, max_value),
            self.y.clamp(min_value, max_value),
            self.z.clamp(min_value, max_value),
        )
    }

    /// Linear interpolation between two vectors.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        self * (1.0 - t) + other * t
    }

    /// Raises each component to a power after clamping at zero.
    pub fn powf(self, power: f64) -> Self {
        Self::new(
            self.x.max(0.0).powf(power),
            self.y.max(0.0).powf(power),
            self.z.max(0.0).powf(power),
        )
    }

    /// Returns the largest component.
    pub fn max_component(self) -> f64 {
        self.x.max(self.y).max(self.z)
    }

    /// Rotates this vector by a quaternion [x, y, z, w].
    pub fn rotate_quaternion(self, quaternion: [f64; 4]) -> Self {
        let [x, y, z, w] = quaternion;
        let norm = (x * x + y * y + z * z + w * w).sqrt();
        if norm <= f64::EPSILON {
            return self;
        }

        let q = Vec3::new(x / norm, y / norm, z / norm);
        let w = w / norm;
        let uv = q.cross(self);
        let uuv = q.cross(uv);
        self + uv * (2.0 * w) + uuv * 2.0
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}
