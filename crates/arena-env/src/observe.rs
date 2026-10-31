use std::array::from_fn;

use arena_sim::{
    action::Facing,
    fighter::{Fighter, MAX_MOVES, State},
    world::{Arena, World},
};

/// Number of values that `observe` writes.
pub const OBS_LEN: usize = 28;

/// Pixels per observed unit. Keeps distances in about 0..3.
const DIST: f32 = 100.0;
/// Pixels per tick per observed unit. Keeps speeds in about -1..1.
const SPEED: f32 = 10.0;
/// Ticks per observed unit, so that times are in seconds.
const TICKS_PER_SEC: u32 = 60;
/// States of a living fighter: free, dash, attack, hurt.
const STATES: usize = 4;

/// Write what fighter `me` sees into `out`. `out` holds at least `OBS_LEN` values.
pub fn observe(world: &World, me: usize, out: &mut [f32]) {
    let foe = &world.fighters[1 - me];
    let me = &world.fighters[me];
    let foe_right = foe.position.x >= me.position.x;

    // Write each group after the one before it.
    let mut len = 0;
    let mut put = |values: &[f32]| {
        out[len..len + values.len()].copy_from_slice(values);
        len += values.len();
    };
    put(&spacing(world.arena, me, foe, foe_right));
    put(&motion(me, foe, foe_right));
    put(&condition(me));
    put(&timers(me));
    debug_assert_eq!(len, OBS_LEN);
}

/// Return the gap, both heights, and the room behind each fighter.
fn spacing(arena: Arena, me: &Fighter, foe: &Fighter, foe_right: bool) -> [f32; 5] {
    // Distance from `x` to the right wall or to the left wall.
    let room = |x: f32, right: bool| {
        if right {
            arena.right_x - x
        } else {
            x - arena.left_x
        }
    };
    [
        (foe.position.x - me.position.x).abs(),
        me.position.y - arena.ground_y,
        foe.position.y - arena.ground_y,
        room(me.position.x, !foe_right),
        room(foe.position.x, foe_right),
    ]
    .map(|v| v / DIST)
}

/// Return both velocities, then 1 or 0 for ground contact and for facing the other fighter.
fn motion(me: &Fighter, foe: &Fighter, foe_right: bool) -> [f32; 8] {
    let dir_sign = if foe_right { 1.0 } else { -1.0 };
    [
        me.velocity.x * dir_sign / SPEED,
        me.velocity.y / SPEED,
        foe.velocity.x * -dir_sign / SPEED,
        foe.velocity.y / SPEED,
        f32::from(me.on_ground),
        f32::from(foe.on_ground),
        f32::from((me.facing == Facing::Right) == foe_right),
        f32::from((foe.facing == Facing::Left) == foe_right),
    ]
}

/// Return the hp, then the state and the move as one-hot. A dead fighter has no state.
fn condition(f: &Fighter) -> [f32; 1 + STATES + MAX_MOVES] {
    let (state, id) = match f.state {
        State::Free => (Some(0), None),
        State::Dash { .. } => (Some(1), None),
        State::Attack { id, .. } => (Some(2), Some(id)),
        State::Hurt { .. } => (Some(3), None),
        State::Dead => (None, None),
    };
    let mut out = [0.0; 1 + STATES + MAX_MOVES];
    out[0] = ratio(f.health, f.stats().max_health);
    out[1..=STATES].copy_from_slice(&one_hot::<STATES>(state));
    out[1 + STATES..].copy_from_slice(&one_hot::<MAX_MOVES>(id));
    out
}

/// Return the seconds until the fighter can act, then the part of each cooldown still left.
///
/// The dash cooldown comes first, then one per move. 1 is just started, 0 is ready.
fn timers(f: &Fighter) -> [f32; 2 + MAX_MOVES] {
    let busy = match f.state {
        State::Dash { left } | State::Hurt { left } => left,
        State::Attack { id, tick, .. } => {
            let m = &f.moves()[id];
            (m.first_active() + m.active + m.recovery).saturating_sub(tick)
        }
        State::Free | State::Dead => 0,
    };
    let mut out = [0.0; 2 + MAX_MOVES];
    out[0] = ratio(busy, TICKS_PER_SEC);
    out[1] = ratio(f.dash_cooldown_left, f.stats().dash_cooldown);
    for (id, m) in f.moves().iter().enumerate() {
        out[2 + id] = ratio(f.cooldown_left[id], m.cooldown);
    }
    out
}

/// Return `N` zeros with a 1 at `slot`. `None` gives all zeros.
fn one_hot<const N: usize>(slot: Option<usize>) -> [f32; N] {
    from_fn(|i| f32::from(slot == Some(i)))
}

/// Return `part / whole`, or 0 if `whole` is 0.
#[expect(clippy::cast_precision_loss, reason = "game counts are small")]
fn ratio(part: u32, whole: u32) -> f32 {
    if whole == 0 {
        0.0
    } else {
        part as f32 / whole as f32
    }
}
