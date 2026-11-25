# Sources

Every outside source that this project uses. [CLAUDE.md](CLAUDE.md) states the rule.

The project license is MIT OR Apache-2.0. A source with no license gives ideas only: no
code from it is copied.

## Papers

- Study used for the agent's grid observation: [NetHack Learning Environment](https://arxiv.org/abs/2006.13760)
  (Küttler et al., 2020). It feeds a grid of glyph IDs, not pixels, to the network.
- Study used for the agent's grid observation: [MinAtar](https://arxiv.org/abs/1903.03176)
  (Young and Tian, 2019). One binary channel per object type.
- Study used for advantage estimation: [High-Dimensional Continuous Control Using
  Generalized Advantage Estimation](https://arxiv.org/abs/1506.02438) (Schulman et al.,
  2015).

## Code

- Study used for the fighter and state design: [2d-souls](https://github.com/rm-a0/2d-souls)
  (MIT). The author's earlier pygame game: one entity base, a flat boss state, timers.
- Study used for the network and the rollout loop: [CleanRL ppo.py](https://github.com/vwxyzjn/cleanrl/blob/master/cleanrl/ppo.py)
  (MIT). Separate actor and critic MLPs, and a rollout buffer of shape steps × envs.
- Study used for the reference PPO and the MultiDiscrete action head:
  [Stable-Baselines3](https://github.com/DLR-RM/stable-baselines3) (MIT). `sb3.py` trains
  it as the baseline. `Agent.act` splits the logits per slot as its
  `MultiCategoricalDistribution` does.

## Art references

- Study used for the renderer's look: [ansenjeo on Instagram](https://www.instagram.com/ansenjeo/).
  Dense letter glyphs for figures over a textured field.
