//! Selection state — bounding box plus a per-pixel mask.

use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub width: u32,
    pub height: u32,
    /// Per-pixel selection mask (alpha = selection strength).
    pub mask: PixelBuffer,
    /// Optional rectangle bounds, used for fast rendering and marching ants.
    pub bounds: Option<Rect>,
    /// Marching ants offset for animation.
    pub ants_phase: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn from_points(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let x = x0.min(x1);
        let y = y0.min(y1);
        let w = (x1 - x0).abs();
        let h = (y1 - y0).abs();
        Self { x, y, w, h }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
    pub fn normalized(&self) -> Self { Self { x: self.x.max(0), y: self.y.max(0), w: self.w, h: self.h } }
}

impl Selection {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
            mask: PixelBuffer::new(w, h),
            bounds: None,
            ants_phase: 0.0,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.mask.resize_canvas(w, h, Rgba::TRANSPARENT);
        self.bounds = None;
    }

    pub fn clear(&mut self) {
        self.mask.clear(Rgba::TRANSPARENT);
        self.bounds = None;
    }

    /// Set a rectangular selection.
    pub fn set_rect(&mut self, r: Rect, feather: f32) {
        self.mask.clear(Rgba::TRANSPARENT);
        let r = r.normalized();
        let mut mask_buf = self.mask.as_mut_slice();
        for y in r.y..(r.y + r.h) {
            for x in r.x..(r.x + r.w) {
                let alpha = if feather <= 0.0 {
                    1.0
                } else {
                    // Distance from nearest edge normalized.
                    let dx = (x - r.x).min(r.x + r.w - 1 - x).max(0) as f32;
                    let dy = (y - r.y).min(r.y + r.h - 1 - y).max(0) as f32;
                    let d = dx.min(dy);
                    (d / feather).min(1.0)
                };
                let idx = (y as u32 * self.width + x as u32) as usize * 4;
                if idx + 3 < mask_buf.len() {
                    mask_buf[idx + 3] = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
        self.bounds = Some(r);
    }

    /// Elliptical selection.
    pub fn set_ellipse(&mut self, r: Rect, feather: f32) {
        self.mask.clear(Rgba::TRANSPARENT);
        let r = r.normalized();
        let cx = r.x as f32 + r.w as f32 * 0.5;
        let cy = r.y as f32 + r.h as f32 * 0.5;
        let rx = r.w as f32 * 0.5;
        let ry = r.h as f32 * 0.5;
        let mut mask_buf = self.mask.as_mut_slice();
        for y in r.y..(r.y + r.h) {
            for x in r.x..(r.x + r.w) {
                let nx = (x as f32 - cx) / rx.max(0.0001);
                let ny = (y as f32 - cy) / ry.max(0.0001);
                let d = (nx * nx + ny * ny).sqrt();
                let alpha = if feather <= 0.0 {
                    if d <= 1.0 { 1.0 } else { 0.0 }
                } else {
                    (1.0 - (d - 1.0) / feather).clamp(0.0, 1.0)
                };
                let idx = (y as u32 * self.width + x as u32) as usize * 4;
                if idx + 3 < mask_buf.len() {
                    mask_buf[idx + 3] = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
        self.bounds = Some(r);
    }

    /// Magic-wand style flood fill of similar colors around a point.
    pub fn magic_wand(&mut self, buf: &PixelBuffer, x: i32, y: i32, tolerance: u8) {
        self.mask.clear(Rgba::TRANSPARENT);
        if x < 0 || y < 0 || x >= buf.width() as i32 || y >= buf.height() as i32 { return; }
        let target = buf.get_pixel(x as u32, y as u32);
        let tol = tolerance as f32 / 255.0;
        let mut mask_buf = self.mask.as_mut_slice();
        let mut visited = vec![false; (buf.width() * buf.height()) as usize];
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if cx < 0 || cy < 0 || cx >= buf.width() as i32 || cy >= buf.height() as i32 { continue; }
            let i = (cy as u32 * buf.width() + cx as u32) as usize;
            if visited[i] { continue; }
            visited[i] = true;
            let p = buf.get_pixel(cx as u32, cy as u32);
            let dr = (p.r - target.r).abs();
            let dg = (p.g - target.g).abs();
            let db = (p.b - target.b).abs();
            let da = (p.a - target.a).abs();
            if dr.max(dg).max(db).max(da) > tol { continue; }
            let idx = i * 4;
            mask_buf[idx + 3] = 255;
            stack.push((cx + 1, cy));
            stack.push((cx - 1, cy));
            stack.push((cx, cy + 1));
            stack.push((cx, cy - 1));
        }
        // Update bounds.
        let mut min_x = i32::MAX; let mut min_y = i32::MAX;
        let mut max_x = i32::MIN; let mut max_y = i32::MIN;
        for y in 0..buf.height() as i32 {
            for x in 0..buf.width() as i32 {
                let idx = ((y as u32 * buf.width() + x as u32) as usize) * 4 + 3;
                if mask_buf[idx] > 0 {
                    min_x = min_x.min(x); min_y = min_y.min(y);
                    max_x = max_x.max(x); max_y = max_y.max(y);
                }
            }
        }
        if min_x != i32::MAX {
            self.bounds = Some(Rect { x: min_x, y: min_y, w: max_x - min_x + 1, h: max_y - min_y + 1 });
        }
    }

    /// Invert selection mask.
    pub fn invert(&mut self) {
        let mut mask_buf = self.mask.as_mut_slice();
        for px in mask_buf.chunks_exact_mut(4) {
            px[3] = 255 - px[3];
        }
    }

    /// Test if a pixel index is selected (>=50%).
    pub fn is_selected_alpha(&self, idx: usize) -> bool {
        let buf = self.mask.as_slice();
        buf.get(idx * 4 + 3).copied().unwrap_or(255) >= 128
    }
}