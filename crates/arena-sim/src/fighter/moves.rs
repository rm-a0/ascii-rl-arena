use crate::physics::{Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub hitbox: Rect,
    pub damage: u32,
    pub knockback: Vec2,
    pub velocity: Option<Vec2>,
}
