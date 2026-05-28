use crate::ui::layout::rect::Rect;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

pub struct Flex {
    pub direction: FlexDirection,
    pub spacing: f64,
    pub padding: f64,
}

impl Flex {
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            spacing: 4.0,
            padding: 0.0,
        }
    }

    pub fn row() -> Self {
        Self::new(FlexDirection::Row)
    }

    pub fn column() -> Self {
        Self::new(FlexDirection::Column)
    }

    pub fn with_spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    pub fn distribute(&self, container: Rect, weights: &[f64]) -> Vec<Rect> {
        if weights.is_empty() {
            return Vec::new();
        }

        let inner = container.shrink(self.padding);
        let total_weight: f64 = weights.iter().sum::<f64>().max(f64::EPSILON);
        let total_spacing = self.spacing * (weights.len() as f64 - 1.0).max(0.0);

        let (axis_size, cross_size) = match self.direction {
            FlexDirection::Row | FlexDirection::RowReverse => (inner.width, inner.height),
            FlexDirection::Column | FlexDirection::ColumnReverse => (inner.height, inner.width),
        };
        let usable = (axis_size - total_spacing).max(0.0);

        let mut out = Vec::with_capacity(weights.len());
        let mut cursor = 0.0;

        let order: Box<dyn Iterator<Item = usize>> = match self.direction {
            FlexDirection::Row | FlexDirection::Column => Box::new(0..weights.len()),
            FlexDirection::RowReverse | FlexDirection::ColumnReverse => {
                Box::new((0..weights.len()).rev())
            }
        };

        let mut placed: Vec<(usize, Rect)> = Vec::with_capacity(weights.len());
        for index in order {
            let size = usable * (weights[index] / total_weight);
            let rect = match self.direction {
                FlexDirection::Row | FlexDirection::RowReverse => {
                    Rect::new(inner.x + cursor, inner.y, size, cross_size)
                }
                FlexDirection::Column | FlexDirection::ColumnReverse => {
                    Rect::new(inner.x, inner.y + cursor, cross_size, size)
                }
            };
            placed.push((index, rect));
            cursor += size + self.spacing;
        }

        out.resize(weights.len(), Rect::ZERO);
        for (index, rect) in placed {
            out[index] = rect;
        }
        out
    }
}
