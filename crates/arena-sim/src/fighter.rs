pub mod moth;
pub mod moves;
pub mod wanderer;

use crate::{
    action::{Action, Facing},
    fighter::moves::{Move, Phase},
    physics::{GRAVITY, Rect, Vec2},
    world::Arena,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Moth,
    Wanderer,
}

impl Kind {
    #[must_use]
    pub const fn stats(self) -> &'static Stats {
        match self {
            Kind::Moth => &moth::STATS,
            Kind::Wanderer => &wanderer::STATS,
        }
    }

    /// Return the move table. A move id indexes it.
    #[must_use]
    pub const fn moves(self) -> &'static [Move] {
        match self {
            Kind::Moth => &moth::MOVES,
            Kind::Wanderer => &wanderer::MOVES,
        }
    }

    /// Return the id of the move that this input starts, if any.
    #[must_use]
    pub fn pick(self, pressed: Action, held: Action, on_ground: bool) -> Option<usize> {
        match self {
            Kind::Moth => moth::pick(pressed, held, on_ground),
            Kind::Wanderer => wanderer::pick(pressed, held, on_ground),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Run, jump and fall under input. The only state that can turn or start something.
    Free,
    Dash {
        left: u32,
    },
    /// `id` indexes the kind's moves. `tick` counts from 0 at the start of the move.
    Attack {
        id: usize,
        tick: u32,
        has_hit: bool,
    },
    Hurt {
        left: u32,
    },
    Dead,
}

/// Numbers that do not change during a fight.
///
/// Speeds are in pixels per tick. Durations are in ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub max_health: u32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub dash_speed: f32,
    pub dash_duration: u32,
    /// Counted from the start of a dash.
    pub dash_cooldown: u32,
    pub body: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fighter {
    pub kind: Kind,
    pub health: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: Facing,
    pub on_ground: bool,
    /// Ticks until the next dash can start.
    pub dash_cooldown_left: u32,
    /// The input of the previous tick, for press and release checks.
    pub last_action: Action,
    pub state: State,
}

impl Fighter {
    /// Return a fighter at full health, standing still. `position` is the feet.
    #[must_use]
    pub fn new(kind: Kind, position: Vec2, facing: Facing) -> Self {
        Self {
            kind,
            health: kind.stats().max_health,
            position,
            velocity: Vec2::ZERO,
            facing,
            on_ground: true,
            dash_cooldown_left: 0,
            last_action: Action::default(),
            state: State::Free,
        }
    }

    #[must_use]
    pub fn stats(&self) -> &'static Stats {
        self.kind.stats()
    }

    #[must_use]
    pub fn moves(&self) -> &'static [Move] {
        self.kind.moves()
    }

    /// Return the body box in the world.
    #[must_use]
    pub fn hurtbox(&self) -> Rect {
        self.stats().body.placed(self.position, self.facing)
    }

    #[must_use]
    pub fn attack_box(&self) -> Option<Rect> {
        match self.state {
            State::Attack { id, tick, has_hit } => {
                let move_ = &self.moves()[id];
                if has_hit || !move_.is_active(tick) {
                    return None;
                }
                let hitbox = move_.hitbox?;
                Some(hitbox.placed(self.position, self.facing))
            }
            _ => None,
        }
    }

    /// Advance the fighter by one tick.
    pub fn update(&mut self, action: Action, arena: Arena) {
        let pressed = action.pressed(self.last_action);
        self.steer(action, pressed);
        if let Some(next) = self.decide(action, pressed) {
            self.enter(next);
        }
        self.behave(action);
        self.integrate(arena);
        self.tick_timers();
        self.last_action = action;
    }

    /// Turn and jump. Neither changes the state, so both combine with any start below.
    fn steer(&mut self, action: Action, pressed: Action) {
        if self.state != State::Free {
            return;
        }
        if let Some(direction) = action.direction {
            self.facing = direction;
        }
        if pressed.jump && self.on_ground {
            self.velocity.y = self.stats().jump_speed;
            self.on_ground = false;
        }
    }

    /// Return the state to switch to, or `None` to stay. The first match wins.
    fn decide(&self, action: Action, pressed: Action) -> Option<State> {
        match self.state {
            State::Free => {
                if let Some(id) = self.kind.pick(pressed, action, self.on_ground) {
                    Some(State::Attack {
                        id,
                        tick: 0,
                        has_hit: false,
                    })
                } else if pressed.dash && self.dash_cooldown_left == 0 {
                    Some(State::Dash {
                        left: self.stats().dash_duration,
                    })
                } else {
                    None
                }
            }
            // Committed: only the timers or the world end these.
            State::Dash { .. } | State::Attack { .. } | State::Hurt { .. } | State::Dead => None,
        }
    }

    /// Switch to `next` and apply its one-time effects.
    fn enter(&mut self, next: State) {
        match next {
            State::Dash { .. } => self.dash_cooldown_left = self.stats().dash_cooldown,
            State::Free | State::Attack { .. } | State::Hurt { .. } | State::Dead => {}
        }
        self.state = next;
    }

    /// Set the horizontal velocity for this tick from the state.
    fn behave(&mut self, action: Action) {
        let sign = self.facing.sign();
        match self.state {
            State::Free => {
                self.velocity.x =
                    action.direction.map_or(0.0, Facing::sign) * self.stats().run_speed;
            }
            State::Dash { .. } => {
                self.velocity.x = sign * self.stats().dash_speed;
                self.velocity.y = 0.0;
            }
            State::Attack { id, tick, .. } => {
                let m = &self.moves()[id];
                self.velocity.x = match m.phase(tick) {
                    Some(Phase::Startup) => sign * m.windup.x,
                    Some(Phase::Active) => sign * m.lunge.x,
                    Some(Phase::Recovery) | None => 0.0,
                };
            }
            State::Hurt { .. } | State::Dead => self.velocity.x = 0.0,
        }
    }

    /// Apply gravity, move, and keep the fighter inside the arena.
    fn integrate(&mut self, arena: Arena) {
        let dashing = matches!(self.state, State::Dash { .. });
        if !dashing {
            self.velocity.y -= GRAVITY;
        }
        self.position += self.velocity;

        if self.position.y <= arena.ground_y {
            self.position.y = arena.ground_y;
            self.velocity.y = 0.0;
            self.on_ground = true;
        }

        let hit_wall = self.position.x <= arena.left_x || self.position.x >= arena.right_x;
        if hit_wall {
            self.position.x = self.position.x.clamp(arena.left_x, arena.right_x);
            self.velocity.x = 0.0;
            if dashing {
                self.state = State::Free;
            }
        }
    }

    /// Count down. A finished dash, move or hurt goes back to `Free`.
    fn tick_timers(&mut self) {
        self.dash_cooldown_left = self.dash_cooldown_left.saturating_sub(1);
        self.state = match self.state {
            State::Dash { left } | State::Hurt { left } if left <= 1 => State::Free,
            State::Dash { left } => State::Dash { left: left - 1 },
            State::Hurt { left } => State::Hurt { left: left - 1 },
            State::Attack { id, tick, has_hit } => {
                if self.moves()[id].phase(tick + 1).is_some() {
                    State::Attack {
                        id,
                        tick: tick + 1,
                        has_hit,
                    }
                } else {
                    State::Free
                }
            }
            State::Free => State::Free,
            State::Dead => State::Dead,
        };
    }
}
