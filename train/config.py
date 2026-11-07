"""All training settings. `tyro.cli(Config)` makes a command-line flag for each field."""

import json
from dataclasses import asdict, dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class PPOSettings:
    """PPO settings that `sb3.py` and `ppo.py` share. Names are SB3's argument names."""

    n_steps: int = 256  # steps per env in one rollout
    batch_size: int = 1024
    n_epochs: int = 4
    learning_rate: float = 3e-4
    gamma: float = 0.99
    gae_lambda: float = 0.95
    ent_coef: float = 0.01
    clip_range: float = 0.2
    vf_coef: float = 0.5
    max_grad_norm: float = 0.5


@dataclass(frozen=True)
class Config:
    """Settings of one training run."""

    seed: int = 0
    steps: int = 5_000_000  # env steps over all envs
    num_envs: int = 16
    hidden: int = 64  # units in each of the 2 hidden layers
    log_interval: int = 5  # updates between log lines
    ppo: PPOSettings = field(default_factory=PPOSettings)

    @property
    def num_updates(self) -> int:
        return self.steps // (self.num_envs * self.ppo.n_steps)

    @property
    def env_seed(self) -> int:
        """Return the first fight seed. Two runs never share a fight."""
        return self.seed * 1_000_000_000

    def save(self, out: str) -> None:
        """Write the settings to `<out>/config.json`, so that the run can be repeated."""
        Path(out).mkdir(parents=True, exist_ok=True)
        Path(out, "config.json").write_text(json.dumps(asdict(self), indent=2))
