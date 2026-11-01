use arena_sim::{
    action::{Action, Facing},
    world::World,
};

use crate::observe::foe_right;

// Slot of each control in a choice.
pub const DIR_DIM: usize = 0;
pub const JUMP_DIM: usize = 1;
pub const DASH_DIM: usize = 2;
pub const ATTACK_DIM: usize = 3;
pub const CAST_DIM: usize = 4;

/// Number of options for each control: stand, toward or away, then four buttons up or down.
pub const ACTION_DIMS: [usize; 5] = [3, 2, 2, 2, 2];

/// Turn the network's choice into the input of fighter `me_id`.
#[must_use]
pub fn decode(choice: [usize; ACTION_DIMS.len()], world: &World, me_id: usize) -> Action {
    let side = |right: bool| if right { Facing::Right } else { Facing::Left };
    let foe_right = foe_right(world, me_id);
    let direction = match choice[DIR_DIM] {
        1 => Some(side(foe_right)),
        2 => Some(side(!foe_right)),
        _ => None,
    };
    Action {
        jump: choice[JUMP_DIM] != 0,
        dash: choice[DASH_DIM] != 0,
        attack: choice[ATTACK_DIM] != 0,
        cast: choice[CAST_DIM] != 0,
        direction,
        ..Action::BASE
    }
}
