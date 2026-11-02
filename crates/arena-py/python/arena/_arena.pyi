"""Type hints for the Rust module `arena._arena`. Keep them the same as `src/lib.rs`."""

import numpy as np
import numpy.typing as npt

OBS_LEN: int
"""Number of values in one observation."""

ACTION_DIMS: list[int]
"""Number of options for each control: direction (stand, toward, away), jump, dash, attack, cast."""

class VecEnv:
    """Fights that step together. A fight that ends starts again in the same step."""

    def __init__(
        self,
        num_envs: int,
        seed: int = 0,
        frame_skip: int = 4,
        hit_dealt: float = 1.0,
        hit_taken: float = -1.0,
        win: float = 10.0,
        loss: float = -10.0,
    ) -> None: ...
    @property
    def num_envs(self) -> int: ...
    def reset(self) -> npt.NDArray[np.float32]:
        """Start a new fight in every env. Return the observations, shape `(num_envs, OBS_LEN)`."""

    def step(
        self, actions: npt.NDArray[np.int64]
    ) -> tuple[npt.NDArray[np.float32], npt.NDArray[np.float32], npt.NDArray[np.bool_]]:
        """Step every fight once. `actions` has shape `(num_envs, len(ACTION_DIMS))`.

        Return the observations, the rewards and the done flags. The observation of a done
        fight is the first one of its next fight.
        """
