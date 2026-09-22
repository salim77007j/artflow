//! Color adjustment filters: brightness/contrast, hue/saturation, levels, curves, invert, grayscale.

use super::Filter;
use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;

pub struct BrightnessContrast { pub brightness: f32, pub contrast: f32 }
impl Filter for BrightnessContrast {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let b = self.brightness.clamp(-1.0, 1.0);
        let c = self.contrast.clamp(-1.0, 1.0);
        let scale = (c + 1.0).max(0.01);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let r = ((p.r - 0.5) * scale + 0.5 + b).clamp(0.0, 1.0);
                let g = ((p.g - 0.5) * scale + 0.5 + b).clamp(0.0, 1.0);
                let bb = ((p.b - 0.5) * scale + 0.5 + b).clamp(0.0, 1.0);
                output.set_pixel(x, y, Rgba::new(r, g, bb, p.a));
            }
        }
    }
}

pub struct HueSaturation { pub hue_shift: f32, pub sat_mult: f32, pub val_mult: f32 }
impl Filter for HueSaturation {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let (h, s, l) = p.to_hsl();
                let nh = (h + self.hue_shift).rem_euclid(1.0);
                let ns = (s * self.sat_mult).clamp(0.0, 1.0);
                let nl = (l * self.val_mult).clamp(0.0, 1.0);
                let c = Rgba::from_hsl(nh, ns, nl);
                output.set_pixel(x, y, Rgba::new(c.r, c.g, c.b, p.a));
            }
        }
    }
}

pub struct Levels { pub black: f32, pub gamma: f32, pub white: f32 }
impl Filter for Levels {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let b = self.black.clamp(0.0, 1.0);
        let w = self.white.clamp(0.001, 1.0).max(b + 0.001);
        let g = self.gamma.max(0.01);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let r = ((p.r - b) / (w - b)).clamp(0.0, 1.0).powf(1.0 / g);
                let gg = ((p.g - b) / (w - b)).clamp(0.0, 1.0).powf(1.0 / g);
                let bb = ((p.b - b) / (w - b)).clamp(0.0, 1.0).powf(1.0 / g);
                output.set_pixel(x, y, Rgba::new(r, gg, bb, p.a));
            }
        }
    }
}

pub struct Invert;
impl Filter for Invert {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                output.set_pixel(x, y, Rgba::new(1.0 - p.r, 1.0 - p.g, 1.0 - p.b, p.a));
            }
        }
    }
}

pub struct Grayscale;
impl Filter for Grayscale {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let l = 0.2126 * p.r + 0.7152 * p.g + 0.0722 * p.b;
                output.set_pixel(x, y, Rgba::new(l, l, l, p.a));
            }
        }
    }
}

pub struct Posterize { pub levels: u32 }
impl Filter for Posterize {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let n = self.levels.max(2) as f32;
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let r = (p.r * n).floor() / (n - 1.0);
                let g = (p.g * n).floor() / (n - 1.0);
                let b = (p.b * n).floor() / (n - 1.0);
                output.set_pixel(x, y, Rgba::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), p.a));
            }
        }
    }
}

pub struct Threshold { pub threshold: f32 }
impl Filter for Threshold {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let t = self.threshold.clamp(0.0, 1.0);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let l = 0.2126 * p.r + 0.7152 * p.g + 0.0722 * p.b;
                let v = if l >= t { 1.0 } else { 0.0 };
                output.set_pixel(x, y, Rgba::new(v, v, v, p.a));
            }
        }
    }
}