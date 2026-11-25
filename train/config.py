"""All training settings. Each field is also a command-line flag."""

import json
from dataclasses import asdict, dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class PPOSettings:
    """PPO settings of `sb3.py` and `ppo.py`. Names are SB3's argument names."""

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

    name: str = "ppo"  # the run goes to runs/<name>/seed<seed>
    seed: int = 0
    steps: int = 5_000_000  # env steps over all envs
    num_envs: int = 16
    hidden: int = 64  # units in each of the 2 hidden layers
    log_interval: int = 5  # updates between log lines
    ppo: PPOSettings = field(default_factory=PPOSettings)

    @property
    def num_updates(self) -> int:
        """Return the number of rollouts in the run."""
        return self.steps // (self.num_envs * self.ppo.n_steps)

    @property
    def env_seed(self) -> int:
        """Return the first fight seed. Two runs never share a fight."""
        return self.seed * 1_000_000_000

    @property
    def out(self) -> str:
        """Return the folder of the run's files."""
        return f"runs/{self.name}/seed{self.seed}"

    def save(self) -> None:
        """Write the settings to `<out>/config.json`."""
        Path(self.out).mkdir(parents=True, exist_ok=True)
        Path(self.out, "config.json").write_text(json.dumps(asdict(self), indent=2))
