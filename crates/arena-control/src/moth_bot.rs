use std::ops::Range;

use arena_sim::{
    action::{Action, Facing},
    fighter::{COUNTER_WINDOW, Fighter, State, moth, moves::Move},
    physics::{Rect, Vec2},
    world::World,
};
use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

use crate::Controller;

/// Tick of the foe's attack on which the bot sees it. Keep it below the foe's startup minus
/// the guard's startup, or the guard comes too late.
const REACTION: u32 = 4;
const GUARD_CHANCE: f64 = 2.0 / 3.0;
const RIPOSTE_CHANCE: f64 = 0.5;
/// Feet closer than this, in pixels, are too close to fight. The bot backs off.
const TOO_CLOSE: f32 = 8.0;
/// Ticks of rest after the bot's own move. This gives the foe a window to attack.
const REST: Range<u32> = 15..40;
/// Ticks that one plan lasts.
const PLAN: Range<u32> = 8..24;

/// The moth's scripted brain. It acts only while its fighter is `Free`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MothBot {
    rng: Pcg32,
    plan: Plan,
    /// Ticks until the next choice. Counts down only while the fighter is `Free`.
    left: u32,
}

/// What the legs do while nothing needs an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Idle,
    Approach,
    BackOff,
}

impl Controller for MothBot {
    /// Rules in priority order. The first rule that gives an action wins.
    fn act(&mut self, world: &World, me: usize) -> Action {
        let foe = &world.fighters[1 - me];
        let me = &world.fighters[me];

        // The sim runs a started move. Release all buttons, or a held one starts it again.
        if me.state != State::Free {
            return Action::BASE;
        }
        if let Some(action) = self.react(me, foe) {
            return action;
        }
        if let Some(action) = self.counter(me, foe) {
            return action;
        }
        if let Some(action) = self.punish(me, foe) {
            return action;
        }
        if self.left > 0 {
            return self.follow(me, foe);
        }
        self.choose(me, foe)
    }
}

impl MothBot {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg32::seed_from_u64(seed),
            plan: Plan::Idle,
            left: 0,
        }
    }

    /// Guard an attack that can reach the bot, on a coin flip. Roll once per attack.
    fn react(&mut self, me: &Fighter, foe: &Fighter) -> Option<Action> {
        let State::Attack { id, tick, .. } = foe.state else {
            return None;
        };
        let m = &foe.moves()[id];
        if tick != REACTION || !can_hit(m, foe.position, foe.facing, me) {
            return None;
        }
        let guard = Action {
            guard: true,
            direction: Some(toward(me, foe)),
            ..Action::BASE
        };
        self.rng.random_bool(GUARD_CHANCE).then_some(guard)
    }

    /// Answer a block once, on the first tick of the counter window: riposte on a coin flip,
    /// or back off.
    fn counter(&mut self, me: &Fighter, foe: &Fighter) -> Option<Action> {
        if me.counter_left != COUNTER_WINDOW {
            return None;
        }
        let to = toward(me, foe);
        if can_hit(&moth::RIPOSTE, me.position, to, foe) && self.rng.random_bool(RIPOSTE_CHANCE) {
            // In the counter window, the attack button starts the riposte.
            let riposte = Action {
                attack: true,
                direction: Some(to),
                ..Action::BASE
            };
            Some(self.commit(riposte))
        } else {
            Some(self.start(Plan::BackOff, me, foe))
        }
    }

    /// Stab a foe that is stuck in its move after a miss and cannot hit the bot.
    fn punish(&mut self, me: &Fighter, foe: &Fighter) -> Option<Action> {
        // In the counter window, attack is the riposte, and `counter` decides that.
        if me.counter_left > 0 {
            return None;
        }
        let State::Attack { id, tick, .. } = foe.state else {
            return None;
        };
        let m = &foe.moves()[id];
        let stuck_for = (m.startup + m.active + m.recovery).saturating_sub(tick);
        let safe = foe
            .strike()
            .is_none_or(|(hitbox, _)| me.hurtbox().is_none_or(|body| !hitbox.intersects(&body)));
        let to = toward(me, foe);
        let lands = tick >= m.startup
            && safe
            && stuck_for > moth::STAB.startup
            && can_hit(&moth::STAB, me.position, to, foe);
        let stab = Action {
            attack: true,
            direction: Some(to),
            ..Action::BASE
        };
        lands.then(|| self.commit(stab))
    }

    /// Pick a move or a new plan by distance. Each arm's range is its chance in percent.
    fn choose(&mut self, me: &Fighter, foe: &Fighter) -> Action {
        let to = toward(me, foe);
        // The direction turns the moth before the move starts, in the same tick.
        let stab = Action {
            attack: true,
            direction: Some(to),
            ..Action::BASE
        };
        let lunge = Action {
            lunge: true,
            direction: Some(to),
            ..Action::BASE
        };
        let roll = self.rng.random_range(0..100);
        if (foe.position.x - me.position.x).abs() < TOO_CLOSE {
            self.start(Plan::BackOff, me, foe)
        } else if can_hit(&moth::STAB, me.position, to, foe) {
            match roll {
                0..50 => self.commit(stab),
                50..75 => self.start(Plan::BackOff, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        } else if can_hit(&moth::LUNGE, me.position, to, foe) {
            match roll {
                0..40 => self.commit(lunge),
                40..80 => self.start(Plan::Approach, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        } else {
            match roll {
                0..80 => self.start(Plan::Approach, me, foe),
                _ => self.start(Plan::Idle, me, foe),
            }
        }
    }

    /// Return `action` and rest after the move it starts.
    fn commit(&mut self, action: Action) -> Action {
        self.plan = Plan::Idle;
        self.left = self.rng.random_range(REST);
        action
    }

    /// Start a plan for a random time and take its first step.
    fn start(&mut self, plan: Plan, me: &Fighter, foe: &Fighter) -> Action {
        self.plan = plan;
        self.left = self.rng.random_range(PLAN);
        self.follow(me, foe)
    }

    /// Take one step of the current plan.
    fn follow(&mut self, me: &Fighter, foe: &Fighter) -> Action {
        self.left = self.left.saturating_sub(1);
        let to = toward(me, foe);
        let direction = match self.plan {
            Plan::Idle => None,
            // Stop in stab range, so that the next choice can stab.
            Plan::Approach if can_hit(&moth::STAB, me.position, to, foe) => {
                self.left = 0;
                None
            }
            Plan::Approach => Some(to),
            // From the foe toward the bot is away from the foe.
            Plan::BackOff => Some(toward(foe, me)),
        };
        Action {
            direction,
            ..Action::BASE
        }
    }
}

/// Return the facing from `me` toward `foe`.
fn toward(me: &Fighter, foe: &Fighter) -> Facing {
    if foe.position.x >= me.position.x {
        Facing::Right
    } else {
        Facing::Left
    }
}

/// Return `true` if move `m`, started at `feet`, can reach `target` where it stands now.
///
/// The hitbox is stretched by the distance that the move travels in startup and active ticks.
#[expect(clippy::cast_precision_loss, reason = "tick counts are small")]
fn can_hit(m: &Move, feet: Vec2, facing: Facing, target: &Fighter) -> bool {
    let (Some(hitbox), Some(body)) = (m.hitbox, target.hurtbox()) else {
        return false;
    };
    let travel = m.windup.x * m.startup as f32 + m.lunge.x * m.active as f32;
    let reach = Rect::new(hitbox.min, Vec2::new(hitbox.max.x + travel, hitbox.max.y));
    reach.placed(feet, facing).intersects(&body)
}
