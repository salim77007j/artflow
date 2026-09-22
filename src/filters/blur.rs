//! Blur / sharpen filters.

use super::Filter;
use crate::render::pixel_buffer::PixelBuffer;

/// Box blur.
pub struct BoxBlur {
    pub radius: u32,
}

impl Filter for BoxBlur {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let r = self.radius as i32;
        let w = input.width() as i32;
        let h = input.height() as i32;
        let mut tmp = PixelBuffer::new(input.width(), input.height());
        // Horizontal pass.
        for y in 0..h {
            let mut sum_r = 0.0; let mut sum_g = 0.0; let mut sum_b = 0.0; let mut sum_a = 0.0;
            let mut count = 0;
            for x in -r..=r {
                let xi = (x).clamp(0, w - 1);
                let p = input.get_pixel(xi as u32, y as u32);
                sum_r += p.r; sum_g += p.g; sum_b += p.b; sum_a += p.a; count += 1;
            }
            for x in 0..w {
                let p_in = input.get_pixel((x + r).min(w - 1) as u32, y as u32);
                let p_out = input.get_pixel((x - r - 1).max(0) as u32, y as u32);
                sum_r += p_in.r - p_out.r;
                sum_g += p_in.g - p_out.g;
                sum_b += p_in.b - p_out.b;
                sum_a += p_in.a - p_out.a;
                tmp.set_pixel(x as u32, y as u32, crate::color::Rgba::new(sum_r / count as f32, sum_g / count as f32, sum_b / count as f32, sum_a / count as f32));
            }
        }
        // Vertical pass.
        for x in 0..w {
            let mut sum_r = 0.0; let mut sum_g = 0.0; let mut sum_b = 0.0; let mut sum_a = 0.0;
            let mut count = 0;
            for y in -r..=r {
                let yi = (y).clamp(0, h - 1);
                let p = tmp.get_pixel(x as u32, yi as u32);
                sum_r += p.r; sum_g += p.g; sum_b += p.b; sum_a += p.a; count += 1;
            }
            for y in 0..h {
                let p_in = tmp.get_pixel(x as u32, (y + r).min(h - 1) as u32);
                let p_out = tmp.get_pixel(x as u32, (y - r - 1).max(0) as u32);
                sum_r += p_in.r - p_out.r;
                sum_g += p_in.g - p_out.g;
                sum_b += p_in.b - p_out.b;
                sum_a += p_in.a - p_out.a;
                output.set_pixel(x as u32, y as u32, crate::color::Rgba::new(sum_r / count as f32, sum_g / count as f32, sum_b / count as f32, sum_a / count as f32));
            }
        }
    }
}

/// Gaussian blur (separable, sigma=1).
pub struct GaussianBlur { pub radius: u32 }
impl Filter for GaussianBlur {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        // Build Gaussian kernel.
        let r = self.radius as i32;
        let sigma = (self.radius as f32 / 2.0).max(0.5);
        let mut kernel = Vec::with_capacity((2 * r + 1) as usize);
        let mut sum = 0.0;
        for i in -r..=r {
            let v = (-((i as f32).powi(2)) / (2.0 * sigma * sigma)).exp();
            kernel.push(v);
            sum += v;
        }
        for v in &mut kernel { *v /= sum; }

        let w = input.width() as i32;
        let h = input.height() as i32;
        let mut tmp = PixelBuffer::new(input.width(), input.height());

        // Horizontal
        for y in 0..h {
            for x in 0..w {
                let mut r0 = 0.0; let mut g0 = 0.0; let mut b0 = 0.0; let mut a0 = 0.0;
                for (ki, k) in kernel.iter().enumerate() {
                    let xi = (x + ki as i32 - r).clamp(0, w - 1);
                    let p = input.get_pixel(xi as u32, y as u32);
                    r0 += p.r * k;
                    g0 += p.g * k;
                    b0 += p.b * k;
                    a0 += p.a * k;
                }
                tmp.set_pixel(x as u32, y as u32, crate::color::Rgba::new(r0, g0, b0, a0));
            }
        }
        // Vertical
        for x in 0..w {
            for y in 0..h {
                let mut r0 = 0.0; let mut g0 = 0.0; let mut b0 = 0.0; let mut a0 = 0.0;
                for (ki, k) in kernel.iter().enumerate() {
                    let yi = (y + ki as i32 - r).clamp(0, h - 1);
                    let p = tmp.get_pixel(x as u32, yi as u32);
                    r0 += p.r * k;
                    g0 += p.g * k;
                    b0 += p.b * k;
                    a0 += p.a * k;
                }
                output.set_pixel(x as u32, y as u32, crate::color::Rgba::new(r0, g0, b0, a0));
            }
        }
    }
}

/// Unsharp-mask sharpening.
pub struct Sharpen { pub amount: f32 }
impl Filter for Sharpen {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        // Simple 3x3 sharpen kernel: 0 -1 0 / -1 5 -1 / 0 -1 0 (clamped).
        let w = input.width() as i32;
        let h = input.height() as i32;
        let k = self.amount.clamp(0.0, 4.0);
        let center = 1.0 + 4.0 * k;
        let edge = -k;
        let mut tmp = PixelBuffer::new(input.width(), input.height());
        for y in 0..h {
            for x in 0..w {
                let mut r0 = 0.0; let mut g0 = 0.0; let mut b0 = 0.0; let mut a0 = 0.0;
                let mut total = 0.0;
                let coords = [
                    (-1i32, 0i32, edge),
                    (1, 0, edge),
                    (0, -1, edge),
                    (0, 1, edge),
                    (0, 0, center),
                ];
                for (dx, dy, w0) in coords {
                    let xi = (x + dx).clamp(0, w - 1);
                    let yi = (y + dy).clamp(0, h - 1);
                    let p = input.get_pixel(xi as u32, yi as u32);
                    r0 += p.r * w0; g0 += p.g * w0; b0 += p.b * w0; a0 += p.a * w0;
                    total += w0;
                }
                if total.abs() > 1e-6 {
                    r0 /= total; g0 /= total; b0 /= total; a0 /= total;
                }
                tmp.set_pixel(x as u32, y as u32, crate::color::Rgba::new(r0.clamp(0.0, 1.0), g0.clamp(0.0, 1.0), b0.clamp(0.0, 1.0), a0.clamp(0.0, 1.0)));
            }
        }
        *output = tmp;
    }
}