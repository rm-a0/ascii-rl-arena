"""Step random actions through the env. Print how a random agent does and the speed."""

import time

import numpy as np

import arena
from train.env import FightStats

NUM_ENVS = 8
STEPS = 1000
SEED = 0


def main() -> None:
    env = arena.VecEnv(NUM_ENVS, seed=SEED)
    obs = env.reset()
    assert obs.shape == (env.num_envs, arena.OBS_LEN)
    rng = np.random.default_rng(SEED)
    stats = FightStats(env.num_envs, window=STEPS)

    start = time.perf_counter()
    for _ in range(STEPS):
        actions = rng.integers(
            0, arena.ACTION_DIMS, size=(env.num_envs, len(arena.ACTION_DIMS))
        )
        obs, rewards, dones = env.step(actions)
        assert obs.shape == (env.num_envs, arena.OBS_LEN)
        assert np.isfinite(obs).all()
        stats.update(rewards, dones)
    seconds = time.perf_counter() - start

    summary = stats.summary()
    print(f"{summary['fights']:.0f} fights finished")
    print(f"mean reward: {summary['reward']:.1f}")
    print(f"mean length: {summary['length']:.0f} steps")
    print(f"win rate: {summary['win_rate']:.0%}")
    print(f"{env.num_envs * STEPS / seconds:,.0f} steps per second")


if __name__ == "__main__":
    main()
