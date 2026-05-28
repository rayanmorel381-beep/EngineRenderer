use crate::ui::layout::rect::Rect;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SplitterAxis {
    Horizontal,
    Vertical,
}

pub struct Splitter {
    pub axis: SplitterAxis,
    pub ratio: f64,
    pub thickness: f64,
    pub min_first: f64,
    pub min_second: f64,
}

impl Splitter {
    pub fn horizontal(ratio: f64) -> Self {
        Self {
            axis: SplitterAxis::Horizontal,
            ratio: ratio.clamp(0.0, 1.0),
            thickness: 4.0,
            min_first: 60.0,
            min_second: 60.0,
        }
    }

    pub fn vertical(ratio: f64) -> Self {
        Self {
            axis: SplitterAxis::Vertical,
            ratio: ratio.clamp(0.0, 1.0),
            thickness: 4.0,
            min_first: 60.0,
            min_second: 60.0,
        }
    }

    pub fn split(&self, container: Rect) -> (Rect, Rect, Rect) {
        match self.axis {
            SplitterAxis::Horizontal => {
                let usable = (container.width - self.thickness).max(0.0);
                let mut first_w = usable * self.ratio;
                first_w = first_w.clamp(self.min_first, (usable - self.min_second).max(0.0));
                let first = Rect::new(container.x, container.y, first_w, container.height);
                let bar = Rect::new(
                    container.x + first_w,
                    container.y,
                    self.thickness,
                    container.height,
                );
                let second = Rect::new(
                    container.x + first_w + self.thickness,
                    container.y,
                    usable - first_w,
                    container.height,
                );
                (first, bar, second)
            }
            SplitterAxis::Vertical => {
                let usable = (container.height - self.thickness).max(0.0);
                let mut first_h = usable * self.ratio;
                first_h = first_h.clamp(self.min_first, (usable - self.min_second).max(0.0));
                let first = Rect::new(container.x, container.y, container.width, first_h);
                let bar = Rect::new(
                    container.x,
                    container.y + first_h,
                    container.width,
                    self.thickness,
                );
                let second = Rect::new(
                    container.x,
                    container.y + first_h + self.thickness,
                    container.width,
                    usable - first_h,
                );
                (first, bar, second)
            }
        }
    }

    pub fn drag(&mut self, container: Rect, pointer_x: f64, pointer_y: f64) {
        match self.axis {
            SplitterAxis::Horizontal => {
                let usable = (container.width - self.thickness).max(1.0);
                self.ratio = ((pointer_x - container.x) / usable).clamp(0.0, 1.0);
            }
            SplitterAxis::Vertical => {
                let usable = (container.height - self.thickness).max(1.0);
                self.ratio = ((pointer_y - container.y) / usable).clamp(0.0, 1.0);
            }
        }
    }
}
