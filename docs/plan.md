# Plan: simulation and controllers

The shape of the modules that are still to write, and the order to write them in. Types
and signatures only: the bodies are yours. A step is done when its check passes.

## Modules

```
crates/arena-sim/src/
  lib.rs           module list
  physics.rs       Vec2, Rect
  action.rs        Facing, Action
  fighter.rs       Kind, Stats, Phase, State, Fighter
  fighter/
    moves.rs       Move
    wanderer.rs    the wanderer's STATS, HURTBOX, MOVES, pick()
    moth.rs        the moth's STATS, HURTBOX, MOVES, pick()
  event.rs         Event
  world.rs         Arena, World
  reward.rs        RewardConfig, reward()

crates/arena-control/src/
  lib.rs           Controller
  rng.rs           Rng, seeded
  moth_bot.rs      MothBot: the moth's scripted brain
  keyboard.rs      Keyboard: a human's brain
```

The sim holds bodies: what a fighter can do and when. A controller holds a brain: what to
do now. They meet only in `Action`.

## Types

```rust
// action.rs
pub struct Action {
    pub direction: Option<Facing>,
    pub jump: bool,
    pub dash: bool,
    pub attack: bool,
    pub guard: bool,
}
impl Action {
    /// Return the buttons that are down now and were up in `last`.
    pub fn pressed(self, last: Action) -> Action;
}

// physics.rs
impl Rect {
    /// Move a box drawn relative to the feet, facing right, into the world.
    pub fn placed(self, feet: Vec2, facing: f32) -> Rect;
}

// fighter/moves.rs
pub struct Move {
    pub startup: u32,             // ticks
    pub active: u32,
    pub recovery: u32,
    pub hitbox: Option<Rect>,     // relative to the feet, facing right; None for a guard
    pub damage: u32,
    pub knockback: Vec2,
    pub windup: Vec2,             // velocity in startup, facing right
    pub lunge: Vec2,              // velocity in active ticks, facing right
    pub guard: bool,              // active ticks block hits from the front
    pub on_block: Option<usize>,  // the move to start after a block: the counter
}
impl Move {
    pub fn phase(&self, tick: u32) -> Option<Phase>;   // None after the last tick
}

// fighter.rs
pub enum Kind { Wanderer, Moth }
pub enum Phase { Startup, Active, Recovery }
pub enum State {
    Free,                                             // run, jump, fall under input
    Dash { left: u32 },
    Attack { id: usize, tick: u32, has_hit: bool },   // id indexes the kind's MOVES
    Hurt { left: u32 },
    Dead,
}
pub struct Fighter {
    pub kind: Kind,
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: Facing,
    pub on_ground: bool,
    pub health: u32,
    pub state: State,
    pub dash_cooldown_left: u32,
    pub invulnerable_left: u32,
    pub last_action: Action,
}
impl Fighter {
    pub fn new(kind: Kind, position: Vec2, facing: Facing) -> Fighter;
    pub fn stats(&self) -> &'static Stats;            // by kind
    pub fn moves(&self) -> &'static [Move];           // by kind
    pub fn update(&mut self, action: Action, arena: Arena);
    fn transition(&mut self, action: Action, pressed: Action);
    fn behave(&mut self, action: Action);
    fn integrate(&mut self, arena: Arena);
    fn tick_timers(&mut self);
    pub fn hurtbox(&self) -> Rect;
    pub fn attack_box(&self) -> Option<(Rect, Phase)>;
    pub fn guarding(&self) -> bool;                   // in a guard move's active ticks
    pub fn take_hit(&mut self, damage: u32, knockback: Vec2);
}

// fighter/moth.rs (wanderer.rs has the same shape)
pub const STATS: Stats;
pub const HURTBOX: Rect;
pub const MOVES: [Move; 4];                           // STAB, LUNGE, GUARD, RIPOSTE
pub fn pick(pressed: Action, held: Action, on_ground: bool) -> Option<usize>;

// event.rs
pub enum Event {
    Hit { attacker: usize, target: usize, damage: u32 },
    Blocked { attacker: usize, target: usize },
    Death { fighter: usize },
    Timeout,
}

// world.rs
pub struct World {
    pub arena: Arena,
    pub fighters: [Fighter; 2],    // 0 is the wanderer, 1 is the moth
    pub tick: u32,
    pub max_ticks: u32,
    pub events: Vec<Event>,        // of the last step
}
impl World {
    pub fn step(&mut self, actions: [Action; 2]) -> &[Event];
    pub fn done(&self) -> bool;
}

// reward.rs
pub struct RewardConfig { pub hit_dealt: f32, pub hit_taken: f32, pub win: f32, pub loss: f32, pub per_tick: f32 }
pub fn reward(events: &[Event], me: usize, cfg: &RewardConfig) -> f32;

// arena-control
pub trait Controller {
    fn act(&mut self, world: &World, me: usize) -> Action;
}
pub struct Rng { /* one u64 */ }
impl Rng {
    pub fn new(seed: u64) -> Rng;
    pub fn chance(&mut self, p: f32) -> bool;
    pub fn range(&mut self, lo: u32, hi: u32) -> u32;
}
pub struct MothBot { rng: Rng, plan: Plan, wait: u32 }
enum Plan { Approach, Keep(f32), Press(Action), Rest }
impl Controller for MothBot { /* ... */ }
```

## One tick

`Fighter::update`:

1. `transition`: in `Free`, `pick` can start a move, a dash or a jump. Only `Free` can turn.
2. `behave`: set the velocity from the state. In a move: `windup` in startup, `lunge` in
   active ticks, slow down in recovery.
3. `integrate`: move, apply gravity when not in a dash, clamp to the arena.
4. `tick_timers`: count down. A finished move, dash or hurt goes back to `Free`.

`World::step`:

1. Clear `events`. Update both fighters.
2. Collect hits with shared borrows. A hit is an active `attack_box` that overlaps the other
   `hurtbox`, with `has_hit` false and the target not invulnerable. Collect first, so that
   the order of the fighters does not matter.
3. Apply the hits. A target that is `guarding` and faces the attacker blocks: no damage,
   push the attacker back, start the target's `on_block` move, push `Event::Blocked`.
   Otherwise: `take_hit`, `Event::Hit`.
4. Health 0: `Dead` and `Event::Death`. `tick == max_ticks`: `Event::Timeout`.

`MothBot::act`:

1. If its own fighter is not `Free`, release all buttons.
2. If the wanderer is in an attack's startup for 12 ticks or more and faces the moth, guard
   with a chance of 0.5.
3. Otherwise choose by distance, with weights: far, lunge or approach; middle, stab, keep
   distance or rest; close, stab, guard or back off.
4. After a move, rest for 15 to 40 ticks.

## Order

| Step | Write | Done when |
| -- | -- | -- |
| 1 | `Action` buttons, `State`, the new `Fighter` fields. Move the dash into `State::Dash` | The preview still runs. A dash lasts `dash_duration` ticks |
| 2 | `Move`, the stab for both kinds, `transition`, `behave`, `tick_timers` | A press of attack starts one stab. The fighter cannot turn in it |
| 3 | Hits, `Hurt`, health, `Event`, death, timeout | Two stabs that overlap take health. The fight ends at 0 |
| 4 | Guard, block, riposte | A stab into a guard deals no damage and starts the riposte |
| 5 | The lunge | Only a new entry in `MOVES` |
| 6 | `Controller`, `Rng`, `MothBot` | Same seed, same fight. The moth guards some attacks, not all |
| 7 | `Keyboard` | You can play the wanderer against the bot in the preview |
| 8 | `RewardConfig`, `reward`, then the observation | The PyO3 env of week 3 can use them |

Tell me when a state exists, and its art frames follow: run, jump, dash, each move's
startup, active and recovery, hurt and death. The renderer then picks frames by `kind` and
`state`. `Fighter::new` with a `Kind` changes the preview's setup, and I update the preview
when it lands.
