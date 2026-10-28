use std::ops::{Add, Mul};

/// An RGB color. Each channel is in 0..=1.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::hex(0x00_0000);

    /// Make a color from a `0xRRGGBB` value.
    #[must_use]
    pub const fn hex(v: u32) -> Rgb {
        Rgb {
            r: ((v >> 16) & 0xff) as f32 / 255.0,
            g: ((v >> 8) & 0xff) as f32 / 255.0,
            b: (v & 0xff) as f32 / 255.0,
        }
    }

    /// Return `self` at `t = 0` and `o` at `t = 1`.
    #[must_use]
    pub fn mix(self, o: Rgb, t: f32) -> Rgb {
        self + (o + self * -1.0) * t
    }

    /// Return the perceived brightness in 0..=1.
    #[must_use]
    pub fn luma(self) -> f32 {
        0.3 * self.r + 0.59 * self.g + 0.11 * self.b
    }

    #[must_use]
    pub fn max(self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    #[must_use]
    pub fn to_u8(self) -> [u8; 3] {
        let q = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        [q(self.r), q(self.g), q(self.b)]
    }
}

impl Add for Rgb {
    type Output = Rgb;
    fn add(self, o: Rgb) -> Rgb {
        Rgb {
            r: self.r + o.r,
            g: self.g + o.g,
            b: self.b + o.b,
        }
    }
}

impl Mul<f32> for Rgb {
    type Output = Rgb;
    fn mul(self, s: f32) -> Rgb {
        Rgb {
            r: self.r * s,
            g: self.g * s,
            b: self.b * s,
        }
    }
}
