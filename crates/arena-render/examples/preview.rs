//! Show the arena with both fighters idle.
//!
//! Run: `scripts/preview.sh` for a kitty window whose font fits the grid, or
//! `cargo run -p arena-render --example preview`. Keys: space pause, q quit.
//!
//! The picture is always `GRID` cells, centered in the terminal. `-- --fit` sets kitty's
//! font size so that the grid fills the window. It needs kitty remote control on a socket,
//! see the script.
//!
//! Write one frame as ANSI text instead: `-- --dump frame.ans --time 1.0`.

use std::{
    io,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use arena_render::{BACKDROP, GRID, Renderer, frame::Frame, term};
use arena_sim::{
    action::Facing,
    fighter::{Fighter, Stats},
    physics::Vec2,
    world::{Arena, World},
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    style::{Color, Style},
};

fn world() -> World {
    let stats = Stats {
        max_health: 5,
        run_speed: 1.5,
        jump_speed: 4.0,
        dash_speed: 4.0,
        dash_duration: 8,
        dash_cooldown: 30,
    };
    let arena = Arena {
        ground_y: 0.0,
        left_x: 0.0,
        right_x: 294.0,
    };
    let player = Fighter::new(stats, Vec2 { x: 112.0, y: 0.0 }, Facing::Right, true);
    let boss = Fighter::new(stats, Vec2 { x: 182.0, y: 0.0 }, Facing::Left, true);
    World::new(arena, [player, boss])
}

/// Return the value after `name` in the arguments.
fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

/// Where the font-fit steps are.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Start,
    Grow,
    Shrink,
    Done,
}

/// Steps kitty's font size by half a point until the terminal is the smallest size that
/// holds the grid.
///
/// It talks to kitty through the socket in `KITTY_LISTEN_ON`, so kitty's replies never mix
/// with the key input on the terminal.
struct Fit {
    step: Step,
    /// The terminal size when the last change was sent, and when.
    last: (u16, u16),
    since: Instant,
    changes: u8,
}

impl Fit {
    fn new() -> Option<Fit> {
        std::env::var_os("KITTY_LISTEN_ON")?;
        Some(Fit {
            step: Step::Start,
            last: (0, 0),
            since: Instant::now(),
            changes: 0,
        })
    }

    fn update(&mut self, now: (u16, u16), grid: (u16, u16)) {
        // Give kitty time to apply the last change. A half point does not always change
        // the number of cells, so wait at most 250 ms.
        if self.step == Step::Done || self.changes >= 80 {
            return;
        }
        if now == self.last && self.since.elapsed() < Duration::from_millis(250) {
            return;
        }
        let fits = now.0 >= grid.0 && now.1 >= grid.1;
        let (next, delta) = match (self.step, fits) {
            (Step::Start | Step::Grow, true) => (Step::Grow, "+0.5"),
            // One step too far: undo it and stop.
            (Step::Grow, false) => (Step::Done, "-0.5"),
            (Step::Start | Step::Shrink, false) => (Step::Shrink, "-0.5"),
            (Step::Shrink, true) | (Step::Done, _) => {
                self.step = Step::Done;
                return;
            }
        };
        let sent = Command::new("kitten")
            .args(["@", "set-font-size", "--", delta])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|mut child| std::thread::spawn(move || child.wait()));
        self.step = if sent.is_ok() { next } else { Step::Done };
        self.last = now;
        self.since = Instant::now();
        self.changes += 1;
    }
}

/// Write `text` at a cell of the terminal buffer in a dim color.
fn note(buf: &mut Buffer, x: u16, y: u16, text: &str) {
    buf.set_string(x, y, text, Style::default().fg(Color::Rgb(111, 154, 134)));
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let world = world();
    let renderer = Renderer::default();
    let mut frame = Frame::default();

    if let Some(path) = arg(&args, "--dump") {
        let t = arg(&args, "--time").map_or(1.0, |a| a.parse().expect("time"));
        renderer.render(&world, t, &mut frame);
        return std::fs::write(path, frame.to_ansi());
    }

    let mut fit = if args.iter().any(|a| a == "--fit") {
        Fit::new()
    } else {
        None
    };
    let [br, bg, bb] = BACKDROP.to_u8();
    let margin = Style::default().bg(Color::Rgb(br, bg, bb));

    let mut term = ratatui::init();
    let start = Instant::now();
    let mut paused_at: Option<f32> = None;
    let result = loop {
        let t = paused_at.unwrap_or_else(|| start.elapsed().as_secs_f32());
        let drawn = term.draw(|f| {
            let area = f.area();
            if let Some(fit) = &mut fit {
                fit.update((area.width, area.height), GRID);
            }
            f.buffer_mut().set_style(area, margin);
            let Some(rect) = term::centered(area, GRID.0, GRID.1) else {
                too_small(f.buffer_mut(), area, GRID);
                return;
            };
            renderer.render(&world, t, &mut frame);
            f.render_widget(&frame, rect);
            if rect.y > area.y {
                note(f.buffer_mut(), area.x, area.y, " space pause · q quit");
            }
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        match event::poll(Duration::from_millis(33)) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(e) => break Err(e),
        }
        let Ok(Event::Key(key)) = event::read() else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Char(' ') => paused_at = paused_at.xor(Some(t)),
            _ => {}
        }
    };
    ratatui::restore();
    result
}

/// Say that the terminal is too small for the grid.
fn too_small(buf: &mut Buffer, area: Rect, grid: (u16, u16)) {
    let lines = [
        format!(
            "The window is {}x{} cells. The arena needs {}x{}.",
            area.width, area.height, grid.0, grid.1
        ),
        "Make the font smaller (ctrl+shift+minus in kitty) or the window bigger.".to_string(),
    ];
    for (dy, line) in (0..).zip(&lines) {
        let w = u16::try_from(line.chars().count()).unwrap_or(u16::MAX);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + (area.height / 2).saturating_sub(1) + dy;
        if y < area.y + area.height {
            note(buf, x, y, line);
        }
    }
}
