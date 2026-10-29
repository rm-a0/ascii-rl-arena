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

/// The input of one tick. A button is `true` while it is down.
///
/// A move button starts its move when the fighter is `Free`. Holding it starts the move
/// again as soon as the fighter is `Free` again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent buttons that combine"
)]
pub struct Action {
    pub jump: bool,
    pub dash: bool,
    pub attack: bool,
    pub lunge: bool,
    pub guard: bool,
    pub cast: bool,
    pub direction: Option<Facing>,
}
