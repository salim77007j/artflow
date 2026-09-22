//! Color utilities, color picker state, palette storage.

use serde::{Deserialize, Serialize};

/// RGBA in 0..=1 linear-ish (we treat as straight-alpha, premultiplied at composite time).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }

    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    pub fn to_rgba8(self) -> (u8, u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    pub fn to_hex(self) -> String {
        let (r, g, b, _) = self.to_rgba8();
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                let a = u8::from_str_radix(&s[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert RGB to HSL.
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let l = (max + min) * 0.5;
        let mut s = 0.0;
        let mut h = 0.0;
        let d = max - min;
        if d > 0.0 {
            s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
            if (max - self.r).abs() < f32::EPSILON {
                h = (self.g - self.b) / d + (if self.g < self.b { 6.0 } else { 0.0 });
            } else if (max - self.g).abs() < f32::EPSILON {
                h = (self.b - self.r) / d + 2.0;
            } else {
                h = (self.r - self.g) / d + 4.0;
            }
            h /= 6.0;
        }
        (h, s, l)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let (r, g, b) = if s.abs() < f32::EPSILON {
            (l, l, l)
        } else {
            let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
            let p = 2.0 * l - q;
            (
                hue_to_rgb(p, q, h + 1.0 / 3.0),
                hue_to_rgb(p, q, h),
                hue_to_rgb(p, q, h - 1.0 / 3.0),
            )
        };
        Self::new(r, g, b, 1.0)
    }

    /// sRGB → linear (gamma 2.2 approx).
    pub fn to_linear(self) -> Self {
        Self::new(self.r.powf(2.2), self.g.powf(2.2), self.b.powf(2.2), self.a)
    }
    pub fn to_srgb(self) -> Self {
        Self::new(self.r.powf(1.0 / 2.2), self.g.powf(1.0 / 2.2), self.b.powf(1.0 / 2.2), self.a)
    }

    /// Lerp toward another RGBA.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 0.5 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

/// Default palette swatches shown in the UI.
pub const DEFAULT_SWATCHES: &[(u8, u8, u8)] = &[
    (0, 0, 0), (255, 255, 255), (128, 128, 128), (192, 192, 192),
    (255, 0, 0), (255, 128, 0), (255, 255, 0), (128, 255, 0),
    (0, 255, 0), (0, 255, 128), (0, 255, 255), (0, 128, 255),
    (0, 0, 255), (128, 0, 255), (255, 0, 255), (255, 0, 128),
    (60, 130, 240), (90, 200, 255), (255, 180, 60),
];

/// User palette + active foreground/background colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorState {
    pub foreground: Rgba,
    pub background: Rgba,
    pub swatches: Vec<Rgba>,
    pub hex_input: String,
    /// Brush settings (mirrored in the right-side panel).
    pub brush_size: f32,
    pub brush_hardness: f32,
    pub brush_opacity: f32,
    pub brush_flow: f32,
    pub brush_spacing: f32,
}

impl Default for ColorState {
    fn default() -> Self {
        Self {
            foreground: Rgba::BLACK,
            background: Rgba::WHITE,
            swatches: DEFAULT_SWATCHES.iter().map(|&(r, g, b)| Rgba::from_rgba8(r, g, b, 255)).collect(),
            hex_input: "#000000".into(),
            brush_size: 20.0,
            brush_hardness: 0.7,
            brush_opacity: 1.0,
            brush_flow: 1.0,
            brush_spacing: 0.1,
        }
    }
}

impl ColorState {
    /// Swap foreground and background.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.foreground, &mut self.background);
    }

    /// Set foreground by hex string. Returns true on parse success.
    pub fn set_fg_hex(&mut self, s: &str) -> bool {
        if let Some(c) = Rgba::from_hex(s) {
            self.foreground = c;
            self.hex_input = s.to_string();
            true
        } else {
            false
        }
    }

    /// Reset to default black/white.
    pub fn reset(&mut self) {
        self.foreground = Rgba::BLACK;
        self.background = Rgba::WHITE;
        self.hex_input = "#000000".into();
    }

    /// Add a swatch (deduped).
    pub fn add_swatch(&mut self, c: Rgba) {
        if !self.swatches.iter().any(|s| (s.r - c.r).abs() < 0.01 && (s.g - c.g).abs() < 0.01 && (s.b - c.b).abs() < 0.01) {
            self.swatches.push(c);
        }
    }
}