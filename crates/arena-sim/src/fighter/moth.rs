use crate::{
    action::Action,
    fighter::{Stats, moves::Move},
    physics::{Rect, Vec2},
};

/// The wings behind the body count too, so a hit that touches them lands.
const BODY: Rect = Rect::new(Vec2::new(-12.0, 0.0), Vec2::new(5.0, 34.0));

pub const STATS: Stats = Stats {
    max_health: 20,
    run_speed: 1.5,
    jump_speed: 4.0,
    dash_speed: 4.0,
    dash_duration: 8,
    dash_cooldown: 30,
    body: BODY,
};

// Indices into `MOVES`.
pub const STAB_ID: usize = 0;
pub const LUNGE_ID: usize = 1;
pub const DIVE_ID: usize = 2;
pub const GUST_ID: usize = 3;

/// A quick thrust that carries the moth about 50 px forward. A foe within about 50 px
/// cannot run from it, only jump or dash.
pub const STAB: Move = Move {
    startup: 12,
    active: 6,
    recovery: 26,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(6.0, 12.0), Vec2::new(28.0, 18.0))),
    lunge: Vec2::new(9.0, 0.0),
    hitstun: 12,
    ..Move::BASE
};

pub const LUNGE: Move = Move {
    startup: 30,
    active: 10,
    recovery: 40,
    damage: 2,
    hitbox: Some(Rect::new(Vec2::new(6.0, 12.0), Vec2::new(28.0, 18.0))),
    knockback: Vec2::new(1.0, 0.0),
    lunge: Vec2::new(5.0, 0.0),
    hitstun: 20,
    ..Move::BASE
};

/// Fly straight up about 52 px, hover, then dive at the foe's feet. The hitbox is the
/// needle, low and in front.
pub const DIVE: Move = Move {
    startup: 30,
    hold: 25,
    active: 14,
    recovery: 40,
    damage: 2,
    hitbox: Some(Rect::new(Vec2::new(4.0, 0.0), Vec2::new(20.0, 14.0))),
    knockback: Vec2::new(1.0, 0.0),
    windup: Vec2::new(0.0, 2.0),
    aim: true,
    hitstun: 20,
    cooldown: 300,
    ..Move::BASE
};

/// A fast wing beat that throws a close foe back. It is quicker than the wanderer's attack.
pub const GUST: Move = Move {
    startup: 6,
    active: 4,
    recovery: 22,
    damage: 1,
    hitbox: Some(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(24.0, 30.0))),
    knockback: Vec2::new(3.0, 2.0),
    hitstun: 12,
    ..Move::BASE
};

pub const MOVES: [Move; 4] = {
    let mut m = [Move::BASE; 4];
    m[STAB_ID] = STAB;
    m[LUNGE_ID] = LUNGE;
    m[DIVE_ID] = DIVE;
    m[GUST_ID] = GUST;
    m
};

/// Return the id of the move that this input starts. Lunge wins, then dive, gust and stab.
/// The moth has no cast, so the cast button starts the dive.
#[must_use]
pub fn pick(action: Action, _on_ground: bool) -> Option<usize> {
    if action.lunge {
        Some(LUNGE_ID)
    } else if action.cast {
        Some(DIVE_ID)
    } else if action.special {
        Some(GUST_ID)
    } else if action.attack {
        Some(STAB_ID)
    } else {
        None
    }
}
