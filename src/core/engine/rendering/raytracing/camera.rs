use super::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub origin: Vec3,
    pub direction: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    focus_distance: f64,
    lens_radius: f64,
    shutter_span: f64,
    motion_vector: Vec3,
}

impl Camera {
    pub fn look_at(origin: Vec3, target: Vec3, up: Vec3, vertical_fov: f64, aspect: f64) -> Self {
        let theta = vertical_fov.to_radians();
        let h = (theta * 0.5).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect * viewport_height;

        let w = (origin - target).normalize();
        let u = up.cross(w).normalize();
        let v = w.cross(u);
        let focus_distance = (origin - target).length();

        let horizontal = u * viewport_width * focus_distance;
        let vertical = v * viewport_height * focus_distance;
        let lower_left_corner = origin - horizontal * 0.5 - vertical * 0.5 - w * focus_distance;

        Self {
            origin,
            direction: (target - origin).normalize(),
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            focus_distance,
            lens_radius: 0.0,
            shutter_span: 0.0,
            motion_vector: Vec3::ZERO,
        }
    }

    pub fn with_physical_lens(mut self, aperture_radius: f64, shutter_span: f64, motion_vector: Vec3) -> Self {
        self.lens_radius = aperture_radius.max(0.0);
        self.shutter_span = shutter_span.max(0.0);
        self.motion_vector = motion_vector;
        self
    }

    pub fn focus_distance(&self) -> f64 {
        self.focus_distance
    }

    pub fn ray(&self, s: f64, t: f64) -> super::primitives::Ray {
        super::primitives::Ray::new(
            self.origin,
            (self.lower_left_corner + self.horizontal * s + self.vertical * t - self.origin)
                .normalize(),
        )
    }

    pub fn ray_with_lens(
        &self,
        s: f64,
        t: f64,
        lens_u: f64,
        lens_v: f64,
        shutter_t: f64,
    ) -> super::primitives::Ray {
        if self.lens_radius <= f64::EPSILON {
            return self.ray(s, t);
        }

        let disk = sample_unit_disk(lens_u, lens_v) * self.lens_radius;
        let lens_offset = self.u * disk.x + self.v * disk.y;
        let motion_offset = self.motion_vector
            * ((shutter_t - 0.5) * self.shutter_span * self.focus_distance.max(1.0));
        let origin = self.origin + lens_offset + motion_offset;
        let target = self.lower_left_corner + self.horizontal * s + self.vertical * t;

        super::primitives::Ray::new(origin, (target - origin).normalize())
    }
}

fn sample_unit_disk(u: f64, v: f64) -> Vec3 {
    let x = u * 2.0 - 1.0;
    let y = v * 2.0 - 1.0;
    let candidate = Vec3::new(x, y, 0.0);
    if candidate.length_squared() <= 1.0 {
        candidate
    } else {
        candidate.normalize() * 0.999
    }
}
