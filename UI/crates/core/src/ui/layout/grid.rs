use crate::ui::layout::rect::Rect;

pub struct Grid {
    pub columns: usize,
    pub rows: usize,
    pub spacing_x: f64,
    pub spacing_y: f64,
    pub padding: f64,
}

impl Grid {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows: rows.max(1),
            spacing_x: 4.0,
            spacing_y: 4.0,
            padding: 0.0,
        }
    }

    pub fn with_spacing(mut self, x: f64, y: f64) -> Self {
        self.spacing_x = x;
        self.spacing_y = y;
        self
    }

    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    pub fn cells(&self, container: Rect) -> Vec<Rect> {
        let inner = container.shrink(self.padding);
        let total_x = self.spacing_x * (self.columns as f64 - 1.0).max(0.0);
        let total_y = self.spacing_y * (self.rows as f64 - 1.0).max(0.0);
        let cell_w = ((inner.width - total_x) / self.columns as f64).max(0.0);
        let cell_h = ((inner.height - total_y) / self.rows as f64).max(0.0);

        let mut out = Vec::with_capacity(self.columns * self.rows);
        for row in 0..self.rows {
            for col in 0..self.columns {
                let x = inner.x + col as f64 * (cell_w + self.spacing_x);
                let y = inner.y + row as f64 * (cell_h + self.spacing_y);
                out.push(Rect::new(x, y, cell_w, cell_h));
            }
        }
        out
    }

    pub fn cell(&self, container: Rect, col: usize, row: usize) -> Rect {
        let inner = container.shrink(self.padding);
        let total_x = self.spacing_x * (self.columns as f64 - 1.0).max(0.0);
        let total_y = self.spacing_y * (self.rows as f64 - 1.0).max(0.0);
        let cell_w = ((inner.width - total_x) / self.columns as f64).max(0.0);
        let cell_h = ((inner.height - total_y) / self.rows as f64).max(0.0);
        let x = inner.x + col as f64 * (cell_w + self.spacing_x);
        let y = inner.y + row as f64 * (cell_h + self.spacing_y);
        Rect::new(x, y, cell_w, cell_h)
    }
}
