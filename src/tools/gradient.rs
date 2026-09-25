//! Gradient fill tool — drags from A to B and fills with a linear gradient.

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use crate::tools::ToolState;
use eframe::egui::Response;

pub fn pointer(doc: &mut Document, layer: &mut PixelBuffer, state: &mut ToolState, pt: (i32, i32), resp: &Response, fg: Rgba, bg: Rgba) {
    let _ = doc;
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() { state.drag_cur = Some(pt); }
    if resp.drag_stopped() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let (x0, y0, x1, y1) = (a.0, a.1, b.0, b.1);
            let dx = (x1 - x0) as f32;
            let dy = (y1 - y0) as f32;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            for y in 0..layer.height() as i32 {
                for x in 0..layer.width() as i32 {
                    let proj = ((x - x0) as f32 * dx + (y - y0) as f32 * dy) / (len * len);
                    let t = proj.clamp(0.0, 1.0);
                    let c = bg.lerp(fg, t);
                    let existing = layer.get_pixel(x as u32, y as u32);
                    let out_a = c.a + existing.a * (1.0 - c.a);
                    if out_a <= 0.0 { continue; }
                    let r = (c.r * c.a + existing.r * existing.a * (1.0 - c.a)) / out_a;
                    let g = (c.g * c.a + existing.g * existing.a * (1.0 - c.a)) / out_a;
                    let b = (c.b * c.a + existing.b * existing.a * (1.0 - c.a)) / out_a;
                    layer.set_pixel(x as u32, y as u32, Rgba::new(r, g, b, out_a));
                }
            }
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
}