# ascii-rl-arena

A 2D side-view boss fight rendered as ASCII art in the terminal. A reinforcement
learning agent controls the boss and learns to fight its opponent.

Nothing is implemented yet. This file lists the decisions made so far and the questions
that are still open.

## Goals

- Learn Rust by writing the simulation by hand.
- Learn reinforcement learning by training a boss.
- Render the fight in the terminal with a high visual quality: animated sprites, color
  and effects, not a plain text UI.
- Make the opponent pluggable: a scripted bot, a human player, or another learning agent
  with a different move set.

## Decisions

| Topic | Decision |
| -- | -- |
| Language | Rust for the simulation and the renderer. Python for training |
| View | 2D side view: one floor, gravity, jump, dash |
| Learner | The boss |
| Opponents | Any fighter can face any other. Each fighter gets its actions from a controller: keyboard, scripted bot or trained policy |
| Order of work | Easiest setup first: a scripted boss against a human, then a learning boss against a scripted player bot. Human opponents and other learning agents come after that |
| First mechanics | Hollow Knight: the Knight as the player and Hornet as the boss |
| Later mechanics | Other games, for example 2D versions of Elden Ring bosses |
| Mechanic structure | No scripting language. Moves and mechanics are Rust code that follows a common structure, so that a new move or mechanic is cheap to add |
| Training | PPO in Python. The Rust simulation is exposed to Python through PyO3 |
| Trained policy in the game | Exported to ONNX and run from Rust |
| Reward | Configurable, not fixed in code |
| Frame size | 240 × 68 terminal cells. Each cell is a half-block character with a foreground and a background color, so a frame is 240 × 136 pixels |
| Output | The current terminal first. Frame composition is separate from output, so that other outputs can be added later |

## Architecture rules

- **The simulation is deterministic.** The same seed and the same actions give the same
  fight. This makes replays and regression tests possible.
- **The simulation does no I/O and no drawing.** Controllers and renderers are outside
  it.
- **The renderer only reads the simulation.** Visual state, such as particles and
  screen shake, never changes the fight.
- **Fighters interact only through shared data and events.** The simulation resolves
  every fight with the same rules.
- **Observations are relative to the observing fighter,** so one policy can play either
  side.
- **A trait is generalized only when two fighters need it.**

## Open questions

**Action space.** One option is to give the two sides different kinds of action:

- The player gets raw buttons every frame. Buttons combine, and hold time matters.
- The boss chooses one committed move, such as a lunge or a jump back, and chooses again
  when the move ends. This is how the Hollow Knight boss AI works.

**Fairness.** Cooldowns and frame data limit each move, but not when the boss uses it.
Hornet in Hollow Knight is fair partly because her AI waits between attacks and reacts
late. A policy with her exact move set can counter each attack on its first frame,
punish every heal, and chain attacks with no gap. Possible limits are an observation
delay, a minimum pause between moves, telegraphs that cannot be cancelled, and
cooldowns per move. These limits can be settings, like the reward.

**Training on human fights.** A recorded fight cannot train the boss as a plain replay,
because the replay does not react to the boss. Options are to start training episodes
from states in a recording, to train a bot that imitates the human (behavior cloning),
and to keep the replay as one opponent in a pool. A boss must handle more than one play
style, so evaluation needs a win rate per opponent.

**Rendering.** The sprite format, the layer structure and the effects are not designed
yet.

**Own window.** A window that the game opens itself would give a fixed font and color
scheme instead of the terminal's.

## AI agents

[CLAUDE.md](CLAUDE.md) states what an AI agent may write in this repository and what it
only reviews.
