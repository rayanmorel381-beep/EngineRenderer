#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub const NONE: Self = Self(0);

    pub fn hash_str(s: &str) -> Self {
        let mut h: u64 = 0xcbf29ce484222325;
        for byte in s.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        Self(h)
    }

    pub fn combine(self, other: WidgetId) -> Self {
        Self(self.0.wrapping_mul(0x100000001b3) ^ other.0)
    }

    pub fn child(self, label: &str) -> Self {
        self.combine(Self::hash_str(label))
    }
}
