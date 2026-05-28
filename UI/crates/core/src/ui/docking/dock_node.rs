use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::Rect;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DockOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub enum DockNode {
    Empty {
        rect: Rect,
    },
    Tabs {
        rect: Rect,
        panels: Vec<WidgetId>,
        active: usize,
    },
    Split {
        rect: Rect,
        orientation: DockOrientation,
        ratio: f64,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

impl DockNode {
    pub fn empty(rect: Rect) -> Self {
        Self::Empty { rect }
    }

    pub fn tabs(rect: Rect, panels: Vec<WidgetId>) -> Self {
        Self::Tabs {
            rect,
            panels,
            active: 0,
        }
    }

    pub fn split(orientation: DockOrientation, ratio: f64, first: DockNode, second: DockNode) -> Self {
        let a = first.rect();
        let b = second.rect();
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = (a.x + a.width).max(b.x + b.width);
        let max_y = (a.y + a.height).max(b.y + b.height);
        let rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
        Self::Split {
            rect,
            orientation,
            ratio: ratio.clamp(0.05, 0.95),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn rect(&self) -> Rect {
        match self {
            Self::Empty { rect } | Self::Tabs { rect, .. } | Self::Split { rect, .. } => *rect,
        }
    }

    pub fn set_rect(&mut self, new_rect: Rect) {
        match self {
            Self::Empty { rect } | Self::Tabs { rect, .. } => *rect = new_rect,
            Self::Split {
                rect,
                orientation,
                ratio,
                first,
                second,
            } => {
                *rect = new_rect;
                match orientation {
                    DockOrientation::Horizontal => {
                        let split_x = new_rect.width * *ratio;
                        first.set_rect(Rect::new(new_rect.x, new_rect.y, split_x, new_rect.height));
                        second.set_rect(Rect::new(
                            new_rect.x + split_x,
                            new_rect.y,
                            new_rect.width - split_x,
                            new_rect.height,
                        ));
                    }
                    DockOrientation::Vertical => {
                        let split_y = new_rect.height * *ratio;
                        first.set_rect(Rect::new(new_rect.x, new_rect.y, new_rect.width, split_y));
                        second.set_rect(Rect::new(
                            new_rect.x,
                            new_rect.y + split_y,
                            new_rect.width,
                            new_rect.height - split_y,
                        ));
                    }
                }
            }
        }
    }

    pub fn for_each_leaf<F: FnMut(Rect, &[WidgetId], usize)>(&self, f: &mut F) {
        match self {
            Self::Empty { .. } => {}
            Self::Tabs { rect, panels, active } => f(*rect, panels, *active),
            Self::Split { first, second, .. } => {
                first.for_each_leaf(f);
                second.for_each_leaf(f);
            }
        }
    }
}
