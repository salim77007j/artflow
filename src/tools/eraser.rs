//! Eraser tool. Same dabs as brush but with transparent ink.

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use crate::tools::brush::{stamp, BrushKind};
use crate::tools::ToolState;
use eframe::egui::Response;

pub fn pointer(
    _doc: &mut Document,
    layer: &mut PixelBuffer,
    state: &mut ToolState,
    pt: (i32, i32),
    resp: &Response,
    _fg: Rgba,
    size: f32,
    hardness: f32,
    opacity: f32,
    spacing: f32,
) {
    let color = Rgba::new(0.0, 0.0, 0.0, 0.0);
    let kind = BrushKind::Round;
    let spacing = spacing.max(0.01);

    if resp.drag_started() {
        state.last_stamp = Some(pt);
        stamp(layer, pt, color, kind, size, hardness, opacity);
        return;
    }
    if resp.dragged() {
        let last = state.last_stamp.unwrap_or(pt);
        let dx = pt.0 - last.0;
        let dy = pt.1 - last.1;
        let dist = ((dx * dx + dy * dy) as f32).sqrt();
        let step = (size * spacing).max(0.5);
        let steps = (dist / step).ceil() as i32;
        for i in 1..=steps {
            let t = i as f32 / steps.max(1) as f32;
            let x = last.0 + (dx as f32 * t) as i32;
            let y = last.1 + (dy as f32 * t) as i32;
            stamp(layer, (x, y), color, kind, size, hardness, opacity);
        }
        state.last_stamp = Some(pt);
        return;
    }
    if resp.drag_stopped() {
        state.last_stamp = None;
    }
}