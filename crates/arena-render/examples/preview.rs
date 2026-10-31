//! Run a fight: you play the wanderer against the moth's bot.
//!
//! Run: `scripts/preview.sh` for a kitty window whose font fits the grid, or
//! `cargo run -p arena-render --example preview`. Keys: esc quit, p pause, r restart,
//! n next arena, - and = slower and faster. Keys and mouse buttons also go to the wanderer's `Keyboard`. Kitty
//! reports key releases, so a held key stays held. In other terminals each press is a
//! tap. `-- --stage lake` starts on the lake. `-- --seed 3` picks the bot's first seed.
//! Each restart adds 1 to it. `-- --speed 0.75` starts at three quarters of full speed.
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

use arena_control::{Controller, keyboard::Keyboard, moth_bot::MothBot};
use arena_render::{BACKDROP, GRID, Renderer, Stage, color::Rgb, frame::Frame, pose_name, term};
use arena_sim::{
    action::Facing,
    fighter::{Fighter, Kind, wanderer},
    physics::Vec2,
    world::{Arena, World},
};
use ratatui::{
    buffer::Buffer,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers, KeyboardEnhancementFlags, ModifierKeyCode, MouseButton, MouseEventKind,
            PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
        },
        execute, terminal,
    },
    layout::Rect,
    style::{Color, Style},
};

/// Simulation ticks per second at full speed.
const TICK_RATE: f64 = 60.0;
/// The slowest and the fastest game speed, and the step of the speed keys.
const SPEEDS: (f64, f64, f64) = (0.25, 2.0, 0.25);
/// Ticks in one fight: 90 seconds.
const MAX_TICKS: u32 = 90 * 60;
/// Ticks to catch up in one frame at most. After a longer stall the fight skips ahead.
const MAX_STEPS_PER_FRAME: u32 = 4;
/// Index of the wanderer in `World::fighters`.
const WANDERER: usize = 0;
/// Index of the moth in `World::fighters`.
const MOTH: usize = 1;

const HUD: Rgb = Rgb::hex(0xe6_e1d2);

fn new_world() -> World {
    let arena = Arena {
        ground_y: 0.0,
        left_x: 0.0,
        right_x: 294.0,
    };
    let player = Fighter::new(Kind::Wanderer, Vec2::new(112.0, 0.0), Facing::Right);
    let boss = Fighter::new(Kind::Moth, Vec2::new(182.0, 0.0), Facing::Left);
    World::new(arena, [player, boss], MAX_TICKS)
}

/// Return the status line: health, what each fighter does, and the result at the end.
fn status(world: &World, seed: u64, pace: f64, fps: u32) -> String {
    let [w, m] = &world.fighters;
    let cast = match w.cooldown_left[wanderer::CAST_ID] {
        0 => "cast ready".to_string(),
        left => format!("cast in {:.1} s", f64::from(left) / TICK_RATE),
    };
    let result = if m.is_dead() {
        "THE WANDERER WINS · "
    } else if w.is_dead() {
        "THE MOTH WINS · "
    } else if world.tick >= world.max_ticks {
        "TIME OUT · "
    } else {
        ""
    };
    format!(
        " {result}wanderer {} hp, {}, {cast} · moth {} hp, {} · seed {seed} · speed {pace}x · fps {fps} ",
        w.health,
        pose_name(w),
        m.health,
        pose_name(m)
    )
}

/// Write `text` into the frame at a cell, over a dark background.
fn hud(frame: &mut Frame, col: i32, row: i32, text: &str) {
    for (i, ch) in (0..).zip(text.chars()) {
        frame.put(col + i, row, ch, HUD);
        frame.set_bg(col + i, row, BACKDROP);
    }
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

/// The fight: the world and the two brains.
struct Fight {
    seed: u64,
    world: World,
    keyboard: Keyboard,
    bot: MothBot,
}

impl Fight {
    fn new(seed: u64) -> Fight {
        Fight {
            seed,
            world: new_world(),
            keyboard: Keyboard::default(),
            bot: MothBot::new(seed),
        }
    }

    /// Start a new fight with the next seed. Held keys stay held.
    fn restart(&mut self) {
        self.seed += 1;
        self.world = new_world();
        self.bot = MothBot::new(self.seed);
    }

    fn step(&mut self) {
        let actions = [
            self.keyboard.act(&self.world, WANDERER),
            self.bot.act(&self.world, MOTH),
        ];
        self.world.step(actions);
    }

    /// Run the ticks that are due by `now` at `pace`, `MAX_STEPS_PER_FRAME` at most. Taps
    /// come up after the first tick.
    fn catch_up(&mut self, next_tick: &mut Instant, now: Instant, pace: f64, taps: &mut Vec<char>) {
        let tick = Duration::from_secs_f64(1.0 / (TICK_RATE * pace));
        let mut steps = 0;
        while *next_tick <= now && !self.world.is_over() {
            if steps == MAX_STEPS_PER_FRAME {
                *next_tick = now;
                break;
            }
            self.step();
            *next_tick += tick;
            steps += 1;
        }
        if steps > 0 {
            for c in taps.drain(..) {
                self.keyboard.key(c, false);
            }
        }
    }
}

/// Counts the frames drawn per second.
struct Meter {
    /// Frames drawn in the last full second.
    fps: u32,
    frames: u32,
    second: Instant,
}

impl Meter {
    fn new() -> Meter {
        Meter {
            fps: 0,
            frames: 0,
            second: Instant::now(),
        }
    }

    /// Count one drawn frame.
    fn count(&mut self) {
        self.frames += 1;
        if self.second.elapsed() >= Duration::from_secs(1) {
            (self.fps, self.frames, self.second) = (self.frames, 0, Instant::now());
        }
    }
}

/// Read every waiting event, so that a burst of mouse moves does not delay the keys.
fn read_events() -> io::Result<Vec<Event>> {
    let mut events = vec![event::read()?];
    while event::poll(Duration::ZERO)? {
        events.push(event::read()?);
    }
    Ok(events)
}

/// Ask the terminal for key releases, Shift on its own, and mouse buttons. Return `true`
/// if key releases arrive. Kitty sends them. Most other terminals send presses only.
fn enable_input() -> bool {
    let mut out = io::stdout();
    let mut releases = terminal::supports_keyboard_enhancement().unwrap_or(false);
    if releases {
        let flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
        releases = execute!(out, PushKeyboardEnhancementFlags(flags)).is_ok();
    }
    // Without mouse capture the preview still runs, and clicks do nothing.
    let _ = execute!(out, EnableMouseCapture);
    releases
}

fn disable_input(releases: bool) {
    let mut out = io::stdout();
    let _ = execute!(out, DisableMouseCapture);
    if releases {
        let _ = execute!(out, PopKeyboardEnhancementFlags);
    }
}

/// Hand a key or mouse event to the wanderer's `Keyboard`. Without key releases, a press
/// is a tap: it goes in `taps` and comes up again after the next tick.
fn feed(keyboard: &mut Keyboard, event: &Event, taps: &mut Vec<char>, releases: bool) {
    match event {
        Event::Key(key) => {
            let down = key.kind != KeyEventKind::Release;
            match key.code {
                // Shift + d arrives as `D`. Lowercase it, so that d stays one key.
                KeyCode::Char(c) => {
                    let c = c.to_ascii_lowercase();
                    keyboard.key(c, down);
                    if down && !releases {
                        taps.push(c);
                    }
                }
                KeyCode::Modifier(ModifierKeyCode::LeftShift | ModifierKeyCode::RightShift) => {
                    keyboard.shift(down);
                }
                _ => {}
            }
        }
        Event::Mouse(mouse) => {
            let (button, down) = match mouse.kind {
                MouseEventKind::Down(b) => (b, true),
                MouseEventKind::Up(b) => (b, false),
                _ => return,
            };
            keyboard.mouse(
                button == MouseButton::Left,
                button == MouseButton::Right,
                down,
            );
        }
        _ => {}
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = arg(&args, "--seed").map_or(0, |a| a.parse().expect("seed"));
    let mut pace: f64 = arg(&args, "--speed").map_or(1.0, |a| a.parse().expect("speed"));
    let mut fight = Fight::new(seed);
    let mut renderer = Renderer::default();
    if arg(&args, "--stage").as_deref() == Some("lake") {
        renderer.stage = Stage::Lake;
    }
    let mut frame = Frame::default();

    if let Some(path) = arg(&args, "--dump") {
        let t = arg(&args, "--time").map_or(1.0, |a| a.parse().expect("time"));
        renderer.render(&fight.world, t, &mut frame);
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
    let releases = enable_input();
    let mut taps = Vec::new();
    let start = Instant::now();
    let mut paused_at: Option<f32> = None;
    let mut next_tick = Instant::now();
    let mut meter = Meter::new();
    let result = 'run: loop {
        let t = paused_at.unwrap_or_else(|| start.elapsed().as_secs_f32());

        // Step the fight at a fixed rate, whatever the frame rate is.
        let now = Instant::now();
        if paused_at.is_some() || fight.world.is_over() {
            next_tick = now;
        }
        fight.catch_up(&mut next_tick, now, pace, &mut taps);

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
            renderer.render(&fight.world, t, &mut frame);
            hud(
                &mut frame,
                0,
                0,
                &status(&fight.world, fight.seed, pace, meter.fps),
            );
            f.render_widget(&frame, rect);
            if rect.y > area.y {
                note(
                    f.buffer_mut(),
                    area.x,
                    area.y,
                    " esc quit · p pause · r restart · n next arena · - = speed",
                );
            }
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        meter.count();
        match event::poll(Duration::from_millis(8)) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(e) => break Err(e),
        }
        let events = match read_events() {
            Ok(events) => events,
            Err(e) => break Err(e),
        };
        for event in &events {
            feed(&mut fight.keyboard, event, &mut taps, releases);
            let Event::Key(key) = event else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let ctrl_c =
                key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Esc => break 'run Ok(()),
                _ if ctrl_c => break 'run Ok(()),
                KeyCode::Char('p') => paused_at = paused_at.xor(Some(t)),
                KeyCode::Char('n') => renderer.stage = renderer.stage.next(),
                KeyCode::Char('r') => fight.restart(),
                KeyCode::Char('-') => pace = (pace - SPEEDS.2).max(SPEEDS.0),
                KeyCode::Char('=' | '+') => pace = (pace + SPEEDS.2).min(SPEEDS.1),
                _ => {}
            }
        }
    };
    disable_input(releases);
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
