//! 32-bit RGBA pixel buffer with helpers for painting, blitting, and resizing.

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelBuffer {
    width: u32,
    height: u32,
    /// RGBA8 interleaved, length = width * height * 4.
    data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), (width * height * 4) as usize);
        Self { width, height, data }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn as_slice(&self) -> &[u8] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut [u8] { &mut self.data }

    pub fn get_pixel(&self, x: u32, y: u32) -> Rgba {
        if x >= self.width || y >= self.height { return Rgba::TRANSPARENT; }
        let idx = (y * self.width + x) as usize * 4;
        Rgba::from_rgba8(self.data[idx], self.data[idx + 1], self.data[idx + 2], self.data[idx + 3])
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, c: Rgba) {
        if x >= self.width || y >= self.height { return; }
        let idx = (y * self.width + x) as usize * 4;
        let (r, g, b, a) = c.to_rgba8();
        self.data[idx] = r;
        self.data[idx + 1] = g;
        self.data[idx + 2] = b;
        self.data[idx + 3] = a;
    }

    pub fn clear(&mut self, c: Rgba) {
        let (r, g, b, a) = c.to_rgba8();
        for px in self.data.chunks_exact_mut(4) {
            px[0] = r; px[1] = g; px[2] = b; px[3] = a;
        }
    }

    /// Resize while preserving existing content (clipped to top-left) and filling new pixels with `bg`.
    pub fn resize_canvas(&mut self, new_w: u32, new_h: u32, bg: Rgba) {
        if new_w == self.width && new_h == self.height { return; }
        let mut new_buf = PixelBuffer::new(new_w, new_h);
        new_buf.clear(bg);
        let copy_w = self.width.min(new_w);
        let copy_h = self.height.min(new_h);
        for y in 0..copy_h {
            for x in 0..copy_w {
                let p = self.get_pixel(x, y);
                new_buf.set_pixel(x, y, p);
            }
        }
        *self = new_buf;
    }

    /// Resize (scaled) to arbitrary dimensions. Uses nearest-neighbour for speed.
    pub fn resize_scaled(&mut self, new_w: u32, new_h: u32) {
        if new_w == self.width && new_h == self.height { return; }
        let mut new_buf = PixelBuffer::new(new_w, new_h);
        let sx = self.width as f32 / new_w.max(1) as f32;
        let sy = self.height as f32 / new_h.max(1) as f32;
        for y in 0..new_h {
            for x in 0..new_w {
                let src_x = (x as f32 * sx) as u32;
                let src_y = (y as f32 * sy) as u32;
                let p = self.get_pixel(src_x.min(self.width - 1), src_y.min(self.height - 1));
                new_buf.set_pixel(x, y, p);
            }
        }
        *self = new_buf;
    }

    /// Apply a convolution-like operation: visit each pixel and call `f`.
    pub fn for_each_pixel(&mut self, mut f: impl FnMut(u32, u32, Rgba) -> Rgba) {
        let w = self.width;
        let h = self.height;
        for y in 0..h {
            for x in 0..w {
                let p = self.get_pixel(x, y);
                let np = f(x, y, p);
                if (np.r - p.r).abs() > 1e-6 || (np.g - p.g).abs() > 1e-6 || (np.b - p.b).abs() > 1e-6 || (np.a - p.a).abs() > 1e-6 {
                    self.set_pixel(x, y, np);
                }
            }
        }
    }
}