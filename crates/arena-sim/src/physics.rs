use std::ops::{Add, AddAssign, Mul, Sub};

use crate::action::Facing;

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

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, v: Vec2) {
        self.x += v.x;
        self.y += v.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}

/// Multiply component-wise.
impl Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * v.x,
            y: self.y * v.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, c: f32) -> Vec2 {
        self * Vec2 { x: c, y: c }
    }
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[must_use]
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub const ZERO: Rect = Rect::new(Vec2::ZERO, Vec2::ZERO);

    #[must_use]
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.min.x < rect.max.x
            && self.max.x > rect.min.x
            && self.min.y < rect.max.y
            && self.max.y > rect.min.y
    }

    /// Move a box drawn relative to the feet, facing right, into the world.
    #[must_use]
    pub fn placed(self, feet: Vec2, facing: Facing) -> Rect {
        let a = feet.x + self.min.x * facing.sign();
        let b = feet.x + self.max.x * facing.sign();
        Rect::new(
            Vec2::new(a.min(b), self.min.y + feet.y),
            Vec2::new(a.max(b), self.max.y + feet.y),
        )
    }
}
