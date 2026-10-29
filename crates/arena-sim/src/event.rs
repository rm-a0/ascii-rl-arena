/// Something that happened between fighters on one tick. Indices point into `World::fighters`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Hit {
        attacker: usize,
        target: usize,
        damage: u32,
    },
    Died {
        fighter: usize,
    },
    Blocked {
        attacker: usize,
        target: usize,
    },
}
