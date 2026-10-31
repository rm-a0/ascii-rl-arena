//! Hand-made ASCII art, read from the text files in `art/`: fighters and backgrounds.
//!
//! An art file has these parts, one per line:
//!
//! - `# ...` is a comment, before the first frame only.
//! - `period 0.6`: seconds per animation frame.
//! - `anchor 5`: the column of the feet in the bottom row of the right-facing frame.
//!   Backgrounds leave it out.
//! - `palette`, then one line per color: a one-letter key, `rrggbb`, and a name.
//! - `frame`, then the glyph rows, a line `--`, and the color rows. A color row holds the
//!   key of each glyph above it. A space in the color rows makes the cell see-through.
//!   A space glyph with a color key is an opaque blank, for holes such as eyes.
//!   An optional second `--` and background rows follow. They set the background color of
//!   a cell by key, for light such as a lantern's glow. A space keeps the background.
//! - `pose stab.active`: the frames after it belong to this pose, until the next `pose`.
//!   Frames before the first `pose` line are the idle pose. The renderer picks a fighter's
//!   pose by its state.
//!
//! Draw every frame facing right. The renderer mirrors it for facing left: `/` and `\`,
//! `(` and `)`, `d` and `b` and the other pairs in `MIRROR` swap. `Y` and `P` swap too, so
//! the corners of `Y88P` keep their shape. Do not use them for anything else.

use crate::{color::Rgb, frame::Frame};

/// Glyph pairs that swap when a frame is mirrored.
const MIRROR: [(char, char); 9] = [
    ('/', '\\'),
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('<', '>'),
    ('d', 'b'),
    ('p', 'q'),
    ('`', '\''),
    ('Y', 'P'),
];

/// One frame: a glyph and a color per cell, row by row from the top.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    /// `None` is a see-through cell.
    pub cells: Vec<Option<(char, Rgb)>>,
    /// Background color per cell. Empty when the frame has no background rows.
    pub bg: Vec<Option<Rgb>>,
    /// The column of the feet in the bottom row.
    pub anchor: usize,
}

impl Grid {
    /// Return the frame flipped left to right.
    #[must_use]
    pub fn mirrored(&self) -> Grid {
        let swap = |c: char| {
            MIRROR
                .iter()
                .find_map(|&(a, b)| (c == a).then_some(b).or((c == b).then_some(a)))
                .unwrap_or(c)
        };
        let cells = self
            .cells
            .chunks(self.w)
            .flat_map(|row| {
                row.iter()
                    .rev()
                    .map(|cell| cell.map(|(c, rgb)| (swap(c), rgb)))
            })
            .collect();
        let bg = self
            .bg
            .chunks(self.w)
            .flat_map(|row| row.iter().rev().copied())
            .collect();
        Grid {
            w: self.w,
            h: self.h,
            cells,
            bg,
            anchor: self.w - 1 - self.anchor,
        }
    }

    /// Draw the frame into `frame` with its feet at cell (`col`, `row`).
    pub fn stamp(&self, frame: &mut Frame, col: i32, row: i32) {
        self.draw(frame, col - self.anchor as i32, row - self.h as i32 + 1);
    }

    /// Draw the frame into `frame` with its top-left corner at cell (`left`, `top`).
    pub fn draw(&self, frame: &mut Frame, left: i32, top: i32) {
        for (i, bg) in self.bg.iter().enumerate() {
            if let Some(rgb) = *bg {
                frame.set_bg(left + (i % self.w) as i32, top + (i / self.w) as i32, rgb);
            }
        }
        for (i, cell) in self.cells.iter().enumerate() {
            if let Some((ch, rgb)) = *cell {
                frame.put(
                    left + (i % self.w) as i32,
                    top + (i / self.w) as i32,
                    ch,
                    rgb,
                );
            }
        }
    }
}

/// Animated art with named poses, each with frames for both facings.
#[derive(Clone, Debug, PartialEq)]
pub struct Art {
    /// The first pose is the idle loop. A background has only that one.
    poses: Vec<Pose>,
    /// Seconds per frame.
    period: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Pose {
    name: String,
    right: Vec<Grid>,
    left: Vec<Grid>,
}

impl Art {
    /// Read an art file. See the module docs for the format.
    ///
    /// # Errors
    ///
    /// Return a message with the line number when the file does not follow the format.
    pub fn parse(text: &str) -> Result<Art, String> {
        let mut period = 0.5;
        let mut anchor = 0;
        let mut palette: Vec<(char, Rgb)> = Vec::new();
        let mut names: Vec<&str> = Vec::new();
        // Each frame with the index of its pose in `names`.
        let mut frames: Vec<(usize, [Vec<&str>; 3])> = Vec::new();
        // 0: header, 1: palette, 2: glyph rows, 3: color rows, 4: background rows.
        let mut part = 0;
        for (n, line) in text.lines().enumerate() {
            let at = |what: &str| format!("line {}: {what}", n + 1);
            // Inside a frame, `#` is a glyph.
            if part < 2 && line.starts_with('#') {
                continue;
            }
            let words: Vec<&str> = line.split_whitespace().collect();
            match (part, words.as_slice()) {
                (_, ["palette"]) => part = 1,
                (_, ["pose", name]) => {
                    names.push(name);
                    part = 0;
                }
                (_, ["frame"]) => {
                    if names.is_empty() {
                        names.push("idle");
                    }
                    frames.push((names.len() - 1, [Vec::new(), Vec::new(), Vec::new()]));
                    part = 2;
                }
                (0, ["period", v]) => period = v.parse().map_err(|_| at("bad period"))?,
                (0, ["anchor", v]) => anchor = v.parse().map_err(|_| at("bad anchor"))?,
                (0 | 1, []) => {}
                (1, [key, hex, ..]) => {
                    let key = key.chars().next().filter(|_| key.len() == 1);
                    let key = key.ok_or_else(|| at("a palette key is one letter"))?;
                    let v = u32::from_str_radix(hex, 16).map_err(|_| at("bad color"))?;
                    palette.push((key, Rgb::hex(v)));
                }
                (2 | 3, ["--"]) => part += 1,
                (2..=4, _) => {
                    if let Some((_, f)) = frames.last_mut() {
                        f[part - 2].push(line);
                    }
                }
                _ => return Err(at("unexpected line")),
            }
        }
        let mut poses: Vec<Pose> = names
            .iter()
            .map(|name| Pose {
                name: (*name).to_string(),
                right: Vec::new(),
                left: Vec::new(),
            })
            .collect();
        for (i, [glyphs, colors, bgs]) in &frames {
            let g = grid(glyphs, colors, bgs, &palette, anchor)?;
            poses[*i].left.push(g.mirrored());
            poses[*i].right.push(g);
        }
        if poses.is_empty() {
            return Err("no frames".into());
        }
        if let Some(empty) = poses.iter().find(|p| p.right.is_empty()) {
            return Err(format!("pose {} has no frames", empty.name));
        }
        Ok(Art { poses, period })
    }

    /// Return the idle frame at time `t`. `facing` is +1 for right and -1 for left.
    #[must_use]
    pub fn frame(&self, t: f32, facing: f32) -> &Grid {
        self.at(&self.poses[0], t, facing)
    }

    /// Return the frame of pose `name` at time `t`. Without that pose, use the pose named
    /// before the dot (`stab` for `stab.active`), then the idle pose.
    #[must_use]
    pub fn pose(&self, name: &str, t: f32, facing: f32) -> &Grid {
        let base = name.split_once('.').map_or(name, |(base, _)| base);
        let find = |n: &str| self.poses.iter().find(|p| p.name == n);
        let pose = find(name).or_else(|| find(base)).unwrap_or(&self.poses[0]);
        self.at(pose, t, facing)
    }

    fn at<'a>(&self, pose: &'a Pose, t: f32, facing: f32) -> &'a Grid {
        let frames = if facing < 0.0 {
            &pose.left
        } else {
            &pose.right
        };
        let i = (t / self.period).max(0.0) as usize % frames.len();
        &frames[i]
    }
}

/// Build one frame from its glyph rows, color rows and background rows.
fn grid(
    glyphs: &[&str],
    colors: &[&str],
    bgs: &[&str],
    palette: &[(char, Rgb)],
    anchor: usize,
) -> Result<Grid, String> {
    let color = |key: char| {
        palette
            .iter()
            .find(|(k, _)| *k == key)
            .map(|&(_, rgb)| rgb)
            .ok_or_else(|| format!("no palette color for key {key:?}"))
    };
    // Trailing blank rows are spacing between frames.
    let h = glyphs
        .iter()
        .rposition(|r| !r.trim().is_empty())
        .map_or(0, |i| i + 1);
    let w = glyphs[..h]
        .iter()
        .map(|r| r.chars().count())
        .max()
        .unwrap_or(0);
    let mut cells = Vec::with_capacity(w * h);
    for (y, row) in glyphs[..h].iter().enumerate() {
        let g: Vec<char> = row.chars().collect();
        let c: Vec<char> = colors.get(y).map_or(Vec::new(), |r| r.chars().collect());
        for x in 0..w {
            let glyph = g.get(x).copied().unwrap_or(' ');
            let key = c.get(x).copied().unwrap_or(' ');
            cells.push(match key {
                ' ' => None,
                key => Some((glyph, color(key)?)),
            });
        }
    }
    let mut bg = Vec::new();
    if bgs.iter().any(|r| !r.trim().is_empty()) {
        for y in 0..h {
            let b: Vec<char> = bgs.get(y).map_or(Vec::new(), |r| r.chars().collect());
            for x in 0..w {
                bg.push(match b.get(x).copied().unwrap_or(' ') {
                    ' ' => None,
                    key => Some(color(key)?),
                });
            }
        }
    }
    if anchor >= w {
        return Err(format!("anchor {anchor} is outside the {w} columns"));
    }
    Ok(Grid {
        w,
        h,
        cells,
        bg,
        anchor,
    })
}
