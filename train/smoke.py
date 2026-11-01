"""Step random actions through the env. Print fight results and the speed."""

import time

import numpy as np

import arena

NUM_ENVS = 64
STEPS = 2_000


def main() -> None:
    env = arena.VecEnv(NUM_ENVS, seed=0)
    rng = np.random.default_rng(0)
    obs = env.reset()
    assert obs.shape == (NUM_ENVS, arena.OBS_LEN)

    returns = np.zeros(NUM_ENVS, dtype=np.float32)
    lengths = np.zeros(NUM_ENVS, dtype=np.int64)
    finished_returns, finished_lengths = [], []

    start = time.perf_counter()
    for _ in range(STEPS):
        actions = rng.integers(0, arena.ACTION_DIMS, size=(NUM_ENVS, len(arena.ACTION_DIMS)))
        obs, rewards, dones = env.step(actions)
        assert np.isfinite(obs).all()
        returns += rewards
        lengths += 1
        finished_returns.extend(returns[dones])
        finished_lengths.extend(lengths[dones])
        returns[dones] = 0.0
        lengths[dones] = 0
    seconds = time.perf_counter() - start

    print(f"{len(finished_returns)} fights finished")
    print(f"mean return {np.mean(finished_returns):.2f}, mean length {np.mean(finished_lengths):.0f} steps")
    print(f"{NUM_ENVS * STEPS / seconds:,.0f} agent steps per second")


if __name__ == "__main__":
    main()
