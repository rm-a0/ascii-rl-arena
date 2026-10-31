pub mod moth;
pub mod moves;
pub mod wanderer;

use crate::{
    action::{Action, Facing},
    fighter::moves::{Move, Phase, Shot, Strike},
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
    pub fn pick(self, action: Action, on_ground: bool) -> Option<usize> {
        match self {
            Kind::Moth => moth::pick(action, on_ground),
            Kind::Wanderer => wanderer::pick(action, on_ground),
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

/// The longest move table of any kind.
pub const MAX_MOVES: usize = 4;
const _: () = assert!(moth::MOVES.len() <= MAX_MOVES && wanderer::MOVES.len() <= MAX_MOVES);

/// What a strike did to the fighter it reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Hit,
    Died,
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
    /// Ticks until each move can start again, by move id.
    pub cooldown_left: [u32; MAX_MOVES],
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
            cooldown_left: [0; MAX_MOVES],
            state: State::Free,
        }
    }

    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.state == State::Dead
    }

    #[must_use]
    pub fn stats(&self) -> &'static Stats {
        self.kind.stats()
    }

    #[must_use]
    pub fn moves(&self) -> &'static [Move] {
        self.kind.moves()
    }

    /// Return the body box in the world. A dead fighter has none, so nothing can hit it.
    #[must_use]
    pub fn hurtbox(&self) -> Option<Rect> {
        (!self.is_dead()).then(|| self.stats().body.placed(self.position, self.facing))
    }

    /// Return the hitbox in the world and the strike it deals, if the move can hit this tick.
    #[must_use]
    pub fn strike(&self) -> Option<(Rect, Strike)> {
        let State::Attack { id, tick, has_hit } = self.state else {
            return None;
        };
        let m = &self.moves()[id];
        if has_hit || !m.is_active(tick) {
            return None;
        }
        let hitbox = m.hitbox?.placed(self.position, self.facing);
        let strike = Strike {
            damage: m.damage,
            // Move data points away from the attacker when it faces right.
            knockback: Vec2::new(m.knockback.x * self.facing.sign(), m.knockback.y),
            hitstun: m.hitstun,
        };
        Some((hitbox, strike))
    }

    /// Return the shot that the move fires this tick. A move fires on its first active tick.
    #[must_use]
    pub fn shot(&self, owner: usize) -> Option<Shot> {
        let State::Attack { id, tick, .. } = self.state else {
            return None;
        };
        let m = &self.moves()[id];
        let projectile = m.projectile?;
        let spawn = Vec2::new(projectile.spawn.x * self.facing.sign(), projectile.spawn.y);
        (tick == m.first_active()).then(|| Shot {
            owner,
            position: self.position + spawn,
            facing: self.facing,
            projectile,
        })
    }

    /// Receive a strike.
    pub fn struck(&mut self, strike: &Strike) -> Outcome {
        self.health = self.health.saturating_sub(strike.damage);
        self.velocity = strike.knockback;
        if strike.knockback.y > 0.0 {
            self.on_ground = false;
        }
        if self.health == 0 {
            self.state = State::Dead;
            Outcome::Died
        } else {
            self.state = State::Hurt {
                left: strike.hitstun,
            };
            Outcome::Hit
        }
    }

    /// Mark the current move as connected, so it cannot hit again.
    pub fn landed(&mut self) {
        if let State::Attack { has_hit, .. } = &mut self.state {
            *has_hit = true;
        }
    }

    /// Advance the fighter by one tick. `foe` is the other fighter's feet.
    pub fn update(&mut self, action: Action, arena: Arena, foe: Vec2) {
        self.steer(action);
        if let Some(next) = self.decide(action) {
            self.enter(next);
        }
        self.behave(action, foe);
        self.integrate(arena);
        self.tick_timers();
    }

    /// Turn and jump. Neither changes the state, so both combine with any start below.
    fn steer(&mut self, action: Action) {
        if self.state != State::Free {
            return;
        }
        if let Some(direction) = action.direction {
            self.facing = direction;
        }
        if action.jump && self.on_ground {
            self.velocity.y = self.stats().jump_speed;
            self.on_ground = false;
        }
    }

    /// Return the state to switch to, or `None` to stay. The first match wins.
    fn decide(&self, action: Action) -> Option<State> {
        match self.state {
            State::Free => {
                let ready = |id: &usize| self.cooldown_left[*id] == 0;
                if let Some(id) = self.kind.pick(action, self.on_ground).filter(ready) {
                    Some(State::Attack {
                        id,
                        tick: 0,
                        has_hit: false,
                    })
                } else if action.dash && self.dash_cooldown_left == 0 {
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
            State::Attack { id, .. } => self.cooldown_left[id] = self.moves()[id].cooldown,
            State::Free | State::Hurt { .. } | State::Dead => {}
        }
        self.state = next;
    }

    /// Set the velocity for this tick from the state.
    #[expect(clippy::cast_precision_loss, reason = "tick counts are small")]
    fn behave(&mut self, action: Action, foe: Vec2) {
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
                let phase = m.phase(tick);
                if phase == Some(Phase::Hold) {
                    self.velocity = Vec2::ZERO;
                    self.facing = if foe.x >= self.position.x {
                        Facing::Right
                    } else {
                        Facing::Left
                    };
                    return;
                }
                if m.aim && phase == Some(Phase::Active) {
                    // Set the flight once. Later active ticks keep it.
                    if tick == m.first_active() {
                        let target = Vec2::new(foe.x, foe.y.min(self.position.y));
                        self.velocity = (target - self.position) * (1.0 / m.active as f32);
                    }
                    return;
                }
                let v = match phase {
                    Some(Phase::Startup) => m.windup,
                    Some(Phase::Active) => m.lunge,
                    Some(Phase::Hold | Phase::Recovery) | None => Vec2::ZERO,
                };
                // A zero x in the air keeps the drift, so an air attack does not stop the jump.
                if self.on_ground || v.x != 0.0 {
                    self.velocity.x = sign * v.x;
                }
                if v.y != 0.0 {
                    self.velocity.y = v.y;
                }
            }
            State::Dead => self.velocity.x = 0.0,
            State::Hurt { .. } => {}
        }
    }

    /// Apply gravity, move, and keep the fighter inside the arena.
    fn integrate(&mut self, arena: Arena) {
        let dashing = matches!(self.state, State::Dash { .. });
        if !self.floats() {
            self.velocity.y -= GRAVITY;
        }
        self.position += self.velocity;

        if self.position.y <= arena.ground_y {
            self.position.y = arena.ground_y;
            self.velocity.y = 0.0;
            self.on_ground = true;
        } else {
            // A move can lift the fighter without a jump.
            self.on_ground = false;
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

    /// Return `true` if gravity does not pull: in a dash, a hold, or an aimed flight.
    fn floats(&self) -> bool {
        match self.state {
            State::Dash { .. } => true,
            State::Attack { id, tick, .. } => {
                let m = &self.moves()[id];
                match m.phase(tick) {
                    Some(Phase::Hold) => true,
                    Some(Phase::Active) => m.aim,
                    Some(Phase::Startup | Phase::Recovery) | None => false,
                }
            }
            State::Free | State::Hurt { .. } | State::Dead => false,
        }
    }

    /// Count down. A finished dash, move or hurt goes back to `Free`.
    fn tick_timers(&mut self) {
        self.dash_cooldown_left = self.dash_cooldown_left.saturating_sub(1);
        for left in &mut self.cooldown_left {
            *left = left.saturating_sub(1);
        }
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
