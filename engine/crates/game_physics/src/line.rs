#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Line {
    Standard { m: f32, b: f32 },
    Vertical { x: f32 },
}

impl Line {
    pub fn intersection(self, other: Line) -> Intersection {
        match (self, other) {
            (l1, l2) if l1 == l2 => Intersection::Line(self),
            (Line::Standard { m: m1, b: b1 }, Line::Standard { m: m2, b: b2 }) if m1 != m2 => {
                let x = (m1 - m2) / (b2 - b1);
                Intersection::Point(x, m1 * x + b1)
            }
            _ => Intersection::None,
        }
    }

    pub fn get_y(self, x: f32) -> Option<f32> {
        self.into_fn().map(|f| f(x))
    }

    pub fn get_x(self, y: f32) -> Option<f32> {
        self.into_fn().map(|f| f(y))
    }

    pub fn into_fn(self) -> Option<impl Fn(f32) -> f32> {
        match self {
            Line::Standard { m, b } => Some(move |x| m * x + b),
            Line::Vertical { .. } => None,
        }
    }

    pub fn into_inverse_fn(self) -> Option<impl Fn(f32) -> f32> {
        if let Line::Standard { m, .. } = self {
            if m == 0.0 {
                return None;
            }
        }

        Some(move |y| match self {
            Line::Standard { m, b } => (y - b) / m,
            Line::Vertical { x } => x,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Intersection {
    None,
    Point(f32, f32),
    Line(Line),
}
