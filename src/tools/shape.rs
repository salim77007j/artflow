//! Shape tools — rectangle, ellipse, line.

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use crate::tools::ToolState;
use eframe::egui::Response;

pub fn pointer_rect(doc: &mut Document, layer: &mut PixelBuffer, state: &mut ToolState, pt: (i32, i32), resp: &Response, fg: Rgba) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() {
        // Live preview: stroke a 1-px border each frame.
        // (Live preview would require storing original state; for simplicity we just commit on release.)
        state.drag_cur = Some(pt);
    }
    if resp.drag_stopped() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
            let (x1, y1) = (a.0.max(b.0), a.1.max(b.1));
            // Outline.
            for x in x0..=x1 {
                plot(layer, x, y0, fg);
                plot(layer, x, y1, fg);
            }
            for y in y0..=y1 {
                plot(layer, x0, y, fg);
                plot(layer, x1, y, fg);
            }
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
    let _ = doc;
}

pub fn pointer_ellipse(doc: &mut Document, layer: &mut PixelBuffer, state: &mut ToolState, pt: (i32, i32), resp: &Response, fg: Rgba) {
    if resp.drag_stopped() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let cx = ((a.0 + b.0) as f32) * 0.5;
            let cy = ((a.1 + b.1) as f32) * 0.5;
            let rx = ((b.0 - a.0).abs() as f32) * 0.5;
            let ry = ((b.1 - a.1).abs() as f32) * 0.5;
            let steps = 360;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let a = t * std::f32::consts::TAU;
                let x = (cx + rx * a.cos()) as i32;
                let y = (cy + ry * a.sin()) as i32;
                plot(layer, x, y, fg);
            }
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() { state.drag_cur = Some(pt); }
    let _ = doc;
}

pub fn pointer_line(doc: &mut Document, layer: &mut PixelBuffer, state: &mut ToolState, pt: (i32, i32), resp: &Response, fg: Rgba) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() { state.drag_cur = Some(pt); }
    if resp.drag_stopped() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            draw_line(layer, a.0, a.1, b.0, b.1, fg);
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
    let _ = doc;
}

fn plot(layer: &mut PixelBuffer, x: i32, y: i32, color: Rgba) {
    if x < 0 || y < 0 || x >= layer.width() as i32 || y >= layer.height() as i32 { return; }
    layer.set_pixel(x as u32, y as u32, color);
}

fn draw_line(layer: &mut PixelBuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        plot(layer, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}