#[derive(Debug, Clone)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Coordinate,
    pub max: Coordinate,
}

impl BoundingBox {

    pub fn zero() -> Self {
        Self::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 0.0 },
        )
    }

    pub fn is_zero(&self) -> bool {
        self.min.x == 0.0 &&
        self.min.y == 0.0 &&
        self.max.x == 0.0 &&
        self.max.y == 0.0
    }

    pub fn new(min: Coordinate, max: Coordinate) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, point: &Coordinate) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }

    fn compute_grid(count: f64) -> (f64, f64) {
        let cols = count.sqrt().ceil();
        let row = (count / cols as f64).ceil();
        (cols, row)
    }

    pub fn divide_bound_box(&self, count: f64) -> Vec<BoundingBox> {
        let (cols, rows) = Self::compute_grid(count);

        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;

        let cell_w = width / cols;
        let cell_h = height / rows;

        let mut boxes = Vec::with_capacity(count as usize);

        for i in 0..(count as usize) {
            let row = i as f64 / cols;
            let col = i as f64 % cols;

            let min = Coordinate::new(
                self.min.x + (col * cell_w),
                self.min.y + (row * cell_h),
            );

            let max = Coordinate::new(
                if col == cols - 1.0 {
                    self.max.x
                } else {
                    self.min.x + ((col + 1.0) * cell_w)
                },
                if row == rows - 1.0 {
                    self.max.y
                } else {
                    self.min.y + ((row + 1.0) * cell_h)
                },
            );

            boxes.push(BoundingBox::new(min, max));
        }

        boxes
    }
}