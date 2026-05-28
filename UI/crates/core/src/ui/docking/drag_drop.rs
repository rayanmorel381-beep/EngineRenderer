use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DropZone {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug, Default)]
pub struct DragDropState {
    pub source: Option<WidgetId>,
    pub target: Option<WidgetId>,
    pub zone: Option<DropZone>,
    pub origin: Vec2,
    pub current: Vec2,
}

impl DragDropState {
    pub fn begin(&mut self, source: WidgetId, origin: Vec2) {
        self.source = Some(source);
        self.target = None;
        self.zone = None;
        self.origin = origin;
        self.current = origin;
    }

    pub fn update(&mut self, current: Vec2) {
        self.current = current;
    }

    pub fn end(&mut self) -> Option<(WidgetId, WidgetId, DropZone)> {
        let result = match (self.source, self.target, self.zone) {
            (Some(s), Some(t), Some(z)) => Some((s, t, z)),
            _ => None,
        };
        self.source = None;
        self.target = None;
        self.zone = None;
        result
    }

    pub fn classify(rect: Rect, point: Vec2) -> Option<DropZone> {
        if !rect.contains(point) {
            return None;
        }
        let cx = rect.center().x;
        let cy = rect.center().y;
        let edge = rect.width.min(rect.height) * 0.25;
        let dx = point.x - cx;
        let dy = point.y - cy;
        if dx.abs() < edge && dy.abs() < edge {
            return Some(DropZone::Center);
        }
        if dx.abs() > dy.abs() {
            if dx < 0.0 {
                Some(DropZone::Left)
            } else {
                Some(DropZone::Right)
            }
        } else if dy < 0.0 {
            Some(DropZone::Top)
        } else {
            Some(DropZone::Bottom)
        }
    }
}
