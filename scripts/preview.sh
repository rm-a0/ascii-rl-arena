#!/bin/sh
# Open the renderer preview in a full-screen kitty window whose font fits the grid.
# Extra arguments go to the preview, for example: scripts/preview.sh --backdrop arena.png
# Remote control only listens on this window's own socket, so the preview can set the font.
cd "$(dirname "$0")/.." || exit 1
exec kitty --start-as=fullscreen \
    -o allow_remote_control=socket-only \
    --listen-on 'unix:@arena-preview-{kitty_pid}' \
    -o background_opacity=1 \
    cargo run -p arena-render --example preview -- --fit "$@"
