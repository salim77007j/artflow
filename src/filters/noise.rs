//! Noise filters: add noise, gaussian noise, median.

use super::Filter;
use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;
use rand::Rng;

pub struct AddNoise { pub amount: f32 }
impl Filter for AddNoise {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let mut rng = rand::thread_rng();
        let a = self.amount.clamp(0.0, 1.0);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let n: f32 = rng.gen_range(-a..a);
                output.set_pixel(x, y, Rgba::new((p.r + n).clamp(0.0, 1.0), (p.g + n).clamp(0.0, 1.0), (p.b + n).clamp(0.0, 1.0), p.a));
            }
        }
    }
}

pub struct GaussianNoise { pub sigma: f32 }
impl Filter for GaussianNoise {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer) {
        let mut rng = rand::thread_rng();
        let s = self.sigma.max(0.0);
        for y in 0..input.height() {
            for x in 0..input.width() {
                let p = input.get_pixel(x, y);
                let nr = gauss(&mut rng, 0.0, s);
                let ng = gauss(&mut rng, 0.0, s);
                let nb = gauss(&mut rng, 0.0, s);
                output.set_pixel(x, y, Rgba::new((p.r + nr).clamp(0.0, 1.0), (p.g + ng).clamp(0.0, 1.0), (p.b + nb).clamp(0.0, 1.0), p.a));
            }
        }
    }
}

fn gauss<R: Rng>(rng: &mut R, mean: f32, sigma: f32) -> f32 {
    // Box-Muller
    let u1: f32 = rng.gen_range(0.0001..1.0);
    let u2: f32 = rng.gen_range(0.0001..1.0);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    mean + sigma * z
}