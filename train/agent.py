from torch import nn

import arena


class Agent(nn.Module):
    def __init__(self, hidden: int) -> None:
        super().__init__()
        state_dim = arena.OBS_LEN
        self.actor = nn.Sequential(
            nn.Linear(state_dim, hidden),
            nn.ReLU(),
            nn.Linear(hidden, hidden),
            nn.ReLU(),
            nn.Linear(hidden, 1),
        )
        self.critic = nn.Sequential(
            nn.Linear(state_dim, hidden),
            nn.ReLU(),
            nn.Linear(hidden, hidden),
            nn.ReLU(),
            nn.Linear(hidden, len(arena.ACTION_DIMS)),
        )
