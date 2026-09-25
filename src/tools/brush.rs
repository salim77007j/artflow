//! Brush engine. Supports round / square stamps with size, hardness, opacity, flow.

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use crate::tools::ToolState;
use eframe::egui::Response;

#[derive(Debug, Clone, Copy)]
pub enum BrushKind { Round, Square }

/// Brush settings.
#[derive(Debug, Clone, Copy)]
pub struct BrushSettings {
    pub size: f32,
    pub hardness: f32, // 0..1
    pub opacity: f32,  // 0..1
    pub flow: f32,     // 0..1
    pub spacing: f32,  // 0..1 of size
    pub kind: BrushKind,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 20.0,
            hardness: 0.7,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.1,
            kind: BrushKind::Round,
        }
    }
}

pub fn pointer(
    _doc: &mut Document,
    layer: &mut PixelBuffer,
    state: &mut ToolState,
    pt: (i32, i32),
    resp: &Response,
    fg: Rgba,
    kind: BrushKind,
    size: f32,
    hardness: f32,
    opacity: f32,
    spacing: f32,
) {
    let spacing = spacing.max(0.01);

    if resp.drag_started() {
        state.last_stamp = Some(pt);
        stamp(layer, pt, fg, kind, size, hardness, opacity);
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
            stamp(layer, (x, y), fg, kind, size, hardness, opacity);
        }
        state.last_stamp = Some(pt);
        return;
    }
    if resp.drag_stopped() {
        state.last_stamp = None;
    }
}

/// Stamp a brush dab at (x, y) onto `layer`.
pub fn stamp(layer: &mut PixelBuffer, pt: (i32, i32), color: Rgba, kind: BrushKind, size: f32, hardness: f32, opacity: f32) {
    let r = (size * 0.5).ceil() as i32;
    let cx = pt.0;
    let cy = pt.1;
    let r2 = (r as f32) * (r as f32);
    let hard_inner = r2 * hardness.clamp(0.0, 1.0);
    for dy in -r..=r {
        for dx in -r..=r {
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= layer.width() as i32 || y >= layer.height() as i32 { continue; }
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 > r2 { continue; }
            let a = match kind {
                BrushKind::Round => {
                    if hardness >= 0.999 {
                        if d2 <= r2 { 1.0 } else { 0.0 }
                    } else if d2 <= hard_inner {
                        1.0
                    } else {
                        let t = (r2 - d2) / (r2 - hard_inner).max(1.0);
                        t.clamp(0.0, 1.0)
                    }
                }
                BrushKind::Square => if d2 <= r2 { 1.0 } else { 0.0 },
            };
            let ink_alpha = color.a * opacity * a;
            if ink_alpha <= 0.0 { continue; }
            let dst = layer.get_pixel(x as u32, y as u32);
            let out_a = ink_alpha + dst.a * (1.0 - ink_alpha);
            if out_a <= 0.0 { continue; }
            let r1 = (color.r * ink_alpha + dst.r * dst.a * (1.0 - ink_alpha)) / out_a;
            let g1 = (color.g * ink_alpha + dst.g * dst.a * (1.0 - ink_alpha)) / out_a;
            let b1 = (color.b * ink_alpha + dst.b * dst.a * (1.0 - ink_alpha)) / out_a;
            layer.set_pixel(x as u32, y as u32, Rgba::new(r1, g1, b1, out_a));
        }
    }
}