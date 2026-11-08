"""Parts that the training programs share."""

from collections import deque

import numpy as np
import numpy.typing as npt


class FightStats:
    """Count the reward, length and result of each fight that ends."""

    def __init__(self, num_envs: int, window: int = 100) -> None:
        # Totals of the fight that runs in each env now.
        self.rewards = np.zeros(num_envs, dtype=np.float32)
        self.lengths = np.zeros(num_envs, dtype=np.int64)
        self.finished_rewards: deque[float] = deque(maxlen=window)
        self.finished_lengths: deque[int] = deque(maxlen=window)
        self.finished_wins: deque[bool] = deque(maxlen=window)
        self.fights = 0

    def update(
        self,
        rewards: npt.NDArray[np.float32],
        dones: npt.NDArray[np.bool_],
    ) -> None:
        """Add one step. Move the fights that ended to the finished lists."""
        self.rewards += rewards
        self.lengths += 1
        self.finished_rewards.extend(self.rewards[dones].tolist())
        self.finished_lengths.extend(self.lengths[dones].tolist())
        self.finished_wins.extend((rewards[dones] > 0).tolist())
        self.fights += int(dones.sum())
        self.rewards[dones] = 0.0
        self.lengths[dones] = 0

    def summary(self) -> dict[str, float]:
        """Return the total fights and the means over the window. Means are NaN before a fight ends."""
        if not self.finished_rewards:
            nan = float("nan")
            return {"fights": 0, "reward": nan, "length": nan, "win_rate": nan}
        return {
            "fights": self.fights,
            "reward": float(np.mean(self.finished_rewards)),
            "length": float(np.mean(self.finished_lengths)),
            "win_rate": float(np.mean(self.finished_wins)),
        }
