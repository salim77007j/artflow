//! Distortion filters: flip, rotate, swirl, wave.

use super::Filter;
use crate::render::pixel_buffer::PixelBuffer;

pub struct FlipVertical;
impl Filter for FlipVertical {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let w = input.width();
        let h = input.height();
        for y in 0..h {
            for x in 0..w {
                output.set_pixel(x, y, input.get_pixel(x, h - 1 - y));
            }
        }
    }
}

pub struct FlipHorizontal;
impl Filter for FlipHorizontal {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let w = input.width();
        let h = input.height();
        for y in 0..h {
            for x in 0..w {
                output.set_pixel(x, y, input.get_pixel(w - 1 - x, y));
            }
        }
    }
}

pub struct Rotate90;
impl Filter for Rotate90 {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let w = input.width();
        let h = input.height();
        for y in 0..h {
            for x in 0..w {
                output.set_pixel(y, w - 1 - x, input.get_pixel(x, y));
            }
        }
    }
}

pub struct Swirl { pub strength: f32 }
impl Filter for Swirl {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let w = input.width() as f32;
        let h = input.height() as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let max_r = (w.min(h)) * 0.5;
        let k = self.strength;
        for y in 0..input.height() {
            for x in 0..input.width() {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let r = (dx * dx + dy * dy).sqrt();
                let t = (1.0 - r / max_r).clamp(0.0, 1.0);
                let a = k * t;
                let cs = a.cos();
                let sn = a.sin();
                let nx = (dx * cs - dy * sn + cx) as i32;
                let ny = (dx * sn + dy * cs + cy) as i32;
                if nx >= 0 && ny >= 0 && nx < input.width() as i32 && ny < input.height() as i32 {
                    output.set_pixel(x, y, input.get_pixel(nx as u32, ny as u32));
                } else {
                    output.set_pixel(x, y, crate::color::Rgba::TRANSPARENT);
                }
            }
        }
    }
}

pub struct Wave { pub amplitude: f32, pub frequency: f32 }
impl Filter for Wave {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let w = input.width() as i32;
        let h = input.height() as i32;
        let amp = self.amplitude;
        let freq = self.frequency;
        for y in 0..h {
            let offset = (amp * (y as f32 * freq).sin()) as i32;
            for x in 0..w {
                let xs = (x + offset).clamp(0, w - 1);
                output.set_pixel(x as u32, y as u32, input.get_pixel(xs as u32, y as u32));
            }
        }
    }
}