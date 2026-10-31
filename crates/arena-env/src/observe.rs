use arena_sim::{
    action::Facing,
    fighter::Fighter,
    world::{Arena, World},
};

/// Number of values that `observe` writes.
pub const OBS_LEN: usize = 13;

/// Pixels per observed unit. Keeps distances in about 0..3.
const DIST: f32 = 100.0;
/// Pixels per tick per observed unit. Keeps speeds in about -1..1.
const SPEED: f32 = 10.0;

/// Write what fighter `me` sees into `out`. `out` holds at least `OBS_LEN` values.
pub fn observe(world: &World, me: usize, out: &mut [f32]) {
    let foe = &world.fighters[1 - me];
    let me = &world.fighters[me];
    let foe_right = foe.position.x >= me.position.x;
    out[0..5].copy_from_slice(&spacing(world.arena, me, foe, foe_right));
    out[5..13].copy_from_slice(&motion(me, foe, foe_right));
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
