#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            width: (max.x - min.x).max(0.0),
            height: (max.y - min.y).max(0.0),
        }
    }

    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x
            && p.x < self.x + self.width
            && p.y >= self.y
            && p.y < self.y + self.height
    }

    pub fn shrink(&self, amount: f64) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - amount * 2.0).max(0.0),
            height: (self.height - amount * 2.0).max(0.0),
        }
    }

    pub fn expand(&self, amount: f64) -> Self {
        self.shrink(-amount)
    }

    pub fn split_left(&self, width: f64) -> (Self, Self) {
        let w = width.clamp(0.0, self.width);
        let left = Self::new(self.x, self.y, w, self.height);
        let right = Self::new(self.x + w, self.y, self.width - w, self.height);
        (left, right)
    }

    pub fn split_right(&self, width: f64) -> (Self, Self) {
        let w = width.clamp(0.0, self.width);
        let left = Self::new(self.x, self.y, self.width - w, self.height);
        let right = Self::new(self.x + self.width - w, self.y, w, self.height);
        (left, right)
    }

    pub fn split_top(&self, height: f64) -> (Self, Self) {
        let h = height.clamp(0.0, self.height);
        let top = Self::new(self.x, self.y, self.width, h);
        let bottom = Self::new(self.x, self.y + h, self.width, self.height - h);
        (top, bottom)
    }

    pub fn split_bottom(&self, height: f64) -> (Self, Self) {
        let h = height.clamp(0.0, self.height);
        let top = Self::new(self.x, self.y, self.width, self.height - h);
        let bottom = Self::new(self.x, self.y + self.height - h, self.width, h);
        (top, bottom)
    }
}
