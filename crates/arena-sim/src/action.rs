/// A horizontal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Right,
    Left,
}

impl Facing {
    /// Return -1 for left and +1 for right.
    #[must_use]
    pub fn sign(self) -> f32 {
        match self {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        }
    }
}

/// The input of one tick. A button is `true` while it is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Action {
    pub jump: bool,
    pub dash: bool,
    pub direction: Option<Facing>,
}
