# CLAUDE.md

Instructions for AI coding agents working in this repository. [README.md](README.md)
holds the design decisions.

## Project

A 2D side-view boss fight rendered as ASCII art in the terminal, with a reinforcement
learning agent as the boss. The simulation and renderer are Rust. Training is PPO in
Python.

This is a learning project for Rust and reinforcement learning. The core is written by
hand, so the agent's main roles are pair programmer, tutor and reviewer.

## Ownership

### Written by hand

- The simulation: world, physics, moves, effects, events and rewards.
- Fighters, their moves and mechanics, and scripted bots.
- Controllers, observations and action spaces.
- The renderer's architecture: sprite format, layers, frame composition and the effect
  system.
- The RL core: the PPO loss, advantage estimation and the reward design.

Do not write or edit this code. Point out the fix instead. Write code here only when the
user asks for one specific edit.

### Written by the agent

- **Art:** sprite frames, backgrounds and color palettes, in the format that the
  renderer's architecture defines.
- **Glue:** CI, `Cargo.toml` content, build scripts, ratatui layout boilerplate, terminal
  output plumbing, PyO3 wrapper plumbing, and training script plumbing (CLI, logging,
  checkpoints, plots, ONNX export).
- **Tests,** when the user asks. Test through public interfaces. If a test finds a bug in
  hand-written code, report it. Do not change that code to make the test pass.
- **Docs, doc comments and comments.**

Say which files you changed. If a file's group is not clear, ask.

## How to help

- When the user is stuck, escalate one step at a time: a hint, then a more specific
  hint, then a generic example of about 10 lines that is not the solution.
- Explain Rust concepts (ownership, borrowing, lifetimes, traits, generics, trait
  objects, iterators, error handling) on the line of code where they matter. Compare with
  Python, C and C++ when that helps.
- Explain RL concepts (PPO, advantages, reward shaping, action masking, self-play) on the
  actual training runs and curves.
- Review with `file:line` references. Name the problem, say why it matters, and give a
  direction. Do not rewrite the code.
- Suggest a crate or tool when it removes boilerplate. Explain the trade-off.

## Design rules to check in reviews

- The simulation is deterministic. No clock, no unseeded randomness, no iteration over a
  `HashMap` where order changes the result.
- The simulation does no I/O and no drawing.
- Renderer state never feeds back into the simulation.
- Fighters interact only through shared data and events.
- A new move uses the common move structure. Flag one-off special cases.
- Observations are relative to the observing fighter.
- A trait is generalized only when two fighters need it.

## Writing

Use the plain voice of ASD-STE100 in docs, doc comments, comments and commit messages:
one idea per sentence, short sentences, active voice, no filler. Start a doc comment with
an imperative summary: "Return the hitbox", not "Returns the hitbox". Write only what is
true now.

## Git

- Read-only commands (`status`, `diff`, `log`, `show`) are always fine.
- Commit only when the user asks. Do not push without asking.
