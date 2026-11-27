#!/usr/bin/env bash
# Install Rust, uv, Python and the packages into your home folder and this folder.
# Needs no admin rights. Needs a C compiler (`cc`): Rust uses it as its linker.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cc > /dev/null; then
    echo "error: no C compiler (cc) found. Install gcc first." >&2
    exit 1
fi

if ! command -v cargo > /dev/null; then
    # Into ~/.cargo and ~/.rustup. rust-toolchain.toml selects the version.
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
        sh -s -- -y --profile minimal --no-modify-path
fi
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v uv > /dev/null; then
    # Into ~/.local/bin.
    curl -LsSf https://astral.sh/uv/install.sh | env UV_NO_MODIFY_PATH=1 sh
fi
export PATH="$HOME/.local/bin:$PATH"

# Python, CPU PyTorch, SB3, and the Rust env built as the module `arena`, all in .venv.
uv sync
uv run python -m train.smoke

echo
echo "Installed. In each new shell, run first:"
echo "  export PATH=\"\$HOME/.cargo/bin:\$HOME/.local/bin:\$PATH\""
