use std::ops::Not;

/// Downward acceleration in pixels per tick per tick.
pub const GRAVITY: f32 = 0.25;

/// A 2D vector in world pixels.
///
/// Positive x points right. Positive y points up.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, v: Vec2) {
        self.x += v.x;
        self.y += v.y;
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}

/// Multiply component-wise.
impl std::ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * v.x,
            y: self.y * v.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, c: f32) -> Vec2 {
        self * Vec2 { x: c, y: c }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[must_use]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.min.x < rect.max.x
            && self.max.x > rect.min.x
            && self.min.y < rect.max.y
            && self.max.y > rect.min.y
    }
}
