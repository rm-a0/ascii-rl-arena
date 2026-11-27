"""Draw the training curves in `runs/`: the mean over seeds and the range of the seeds."""

import csv
import warnings
from dataclasses import dataclass
from pathlib import Path

import matplotlib

matplotlib.use("Agg")  # write files, open no window

import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
import tyro  # noqa: E402
from matplotlib.ticker import PercentFormatter  # noqa: E402

# Before the first fight ends, every seed is NaN. Leave those points empty.
warnings.filterwarnings("ignore", "Mean of empty slice|All-NaN slice")

# SB3's column names, as ours.
SB3_COLUMNS = {
    "time/total_timesteps": "steps",
    "fights/win_rate": "win_rate",
    "fights/reward": "reward",
    "train/entropy_loss": "entropy",
    "train/clip_fraction": "clip_frac",
}

# Label and color of each method, in a fixed order.
METHODS = {
    "ppo": ("PPO", "#257de0"),
    "ac": ("Actor-critic", "#f47230"),
    "sb3": ("SB3 PPO", "#00b894"),
}

# Column, title and y label of each figure.
FIGURES = [
    ("win_rate", "Win rate against the moth", "win rate (last 100 fights)"),
    ("reward", "Fight reward", "mean reward (last 100 fights)"),
    ("entropy", "Policy entropy", "entropy (nats)"),
    ("clip_frac", "Share of clipped samples", "clip fraction"),
]

TEXT = "#191919"
MUTED = "#666666"
GRID = "#e5e5e5"
POINTS = 300  # points on the common step grid


@dataclass(frozen=True)
class PlotConfig:
    """Settings of the figures."""

    runs: str = "runs"  # folder with one `<name>/seed<k>` folder per run
    out: str = "runs/plots"


def read(path: Path) -> dict[str, np.ndarray]:
    """Return each column of a `progress.csv`, with our names. Empty cells are NaN."""
    with path.open() as file:
        rows = list(csv.DictReader(file))
    columns = {
        SB3_COLUMNS.get(name, name): np.array(
            [float(row[name]) if row[name] else np.nan for row in rows]
        )
        for name in rows[0]
    }
    if "train/entropy_loss" in rows[0]:
        columns["entropy"] = -columns["entropy"]  # SB3 logs the negative entropy
    return columns


def curves(
    runs: list[dict[str, np.ndarray]], column: str, grid: np.ndarray
) -> np.ndarray:
    """Return one row per run: `column` on the step grid, NaN where it is unknown."""
    rows = []
    for run in runs:
        known = np.isfinite(run[column])
        steps, values = run["steps"][known], run[column][known]
        rows.append(np.interp(grid, steps, values, left=np.nan, right=np.nan))
    return np.array(rows)


def label_ends(ax: plt.Axes, ends: list[tuple[float, str, str]]) -> None:
    """Write each label at the right of its line end, pushed apart if they overlap."""
    low, high = ax.get_ylim()
    gap = (high - low) * 0.06
    placed: list[float] = []
    for y, text, color in sorted(ends):
        if placed and y - placed[-1] < gap:
            y = placed[-1] + gap
        placed.append(y)
        ax.annotate(
            text, (1.01, y), xycoords=("axes fraction", "data"),
            color=TEXT, fontsize=9, va="center",
        )  # fmt: skip
        ax.plot([1.0], [y], marker="o", markersize=4, color=color,
                transform=ax.get_yaxis_transform(), clip_on=False)  # fmt: skip


def draw(
    methods: dict[str, list[dict[str, np.ndarray]]],
    column: str,
    title: str,
    ylabel: str,
    path: Path,
) -> None:
    """Draw one figure: a mean line and a seed band for each method."""
    shown = {name: runs for name, runs in methods.items() if column in runs[0]}
    if not shown:
        return
    end = min(run["steps"][-1] for runs in shown.values() for run in runs)
    grid = np.linspace(0, end, POINTS)

    fig, ax = plt.subplots(figsize=(6.4, 3.6), dpi=200)
    ends = []
    for name, runs in shown.items():
        label, color = METHODS.get(name, (name, MUTED))
        values = curves(runs, column, grid)
        x = grid / 1e6
        mean = np.nanmean(values, axis=0)
        ax.fill_between(
            x, np.nanmin(values, axis=0), np.nanmax(values, axis=0),
            color=color, alpha=0.15, linewidth=0,
        )  # fmt: skip
        seeds = f"{len(runs)} seed" + ("s" if len(runs) > 1 else "")
        ax.plot(x, mean, color=color, linewidth=1.5, label=f"{label} ({seeds})")
        last = mean[np.isfinite(mean)][-1]
        ends.append((last, label, color))

    ax.set_title(title, loc="left", color=TEXT, fontsize=11)
    ax.set_xlabel("env steps (millions)", color=MUTED, fontsize=9)
    ax.set_ylabel(ylabel, color=MUTED, fontsize=9)
    ax.set_xlim(0, end / 1e6)
    if column == "win_rate":
        ax.set_ylim(0, 1)
        ax.yaxis.set_major_formatter(PercentFormatter(1))
    ax.grid(color=GRID, linewidth=0.8)
    ax.set_axisbelow(True)
    ax.tick_params(colors=MUTED, labelsize=8, length=0)
    for side in ("top", "right", "left", "bottom"):
        ax.spines[side].set_visible(side == "bottom")
    ax.spines["bottom"].set_color(MUTED)
    ax.legend(frameon=False, fontsize=8, labelcolor=TEXT, loc="best")
    label_ends(ax, ends)
    fig.tight_layout()
    fig.savefig(path)
    plt.close(fig)


def main() -> None:
    cfg = tyro.cli(PlotConfig, description=__doc__)
    methods: dict[str, list[dict[str, np.ndarray]]] = {}
    for path in sorted(Path(cfg.runs).glob("*/seed*/progress.csv")):
        methods.setdefault(path.parent.parent.name, []).append(read(path))
    # The fixed order first, then any other method.
    methods = {
        name: methods[name]
        for name in [*METHODS, *sorted(methods)]
        if name in methods
    }

    out = Path(cfg.out)
    out.mkdir(parents=True, exist_ok=True)
    for column, title, ylabel in FIGURES:
        draw(methods, column, title, ylabel, out / f"{column}.png")
    print(f"wrote {len(FIGURES)} figures to {out}/")


if __name__ == "__main__":
    main()
