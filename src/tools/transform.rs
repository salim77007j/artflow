//! Move / crop tools.

use crate::document::Document;
use crate::tools::ToolState;
use eframe::egui::Response;

pub fn pointer_move(_doc: &mut Document, state: &mut ToolState, pt: (i32, i32), resp: &Response) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() {
        state.drag_cur = Some(pt);
    }
    if resp.drag_released() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let dx = b.0 - a.0;
            let dy = b.1 - a.1;
            // For a real move we'd shift the layer pixels — here we just translate the transform overlay.
            // The actual "move layer contents" is exposed as an Image > Transform menu.
            let _ = (dx, dy);
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
}

pub fn pointer_crop(doc: &mut Document, state: &mut ToolState, pt: (i32, i32), resp: &Response) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() { state.drag_cur = Some(pt); }
    if resp.drag_released() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let x = a.0.min(b.0).max(0) as u32;
            let y = a.1.min(b.1).max(0) as u32;
            let w = (b.0 - a.0).abs() as u32;
            let h = (b.1 - a.1).abs() as u32;
            if w > 0 && h > 0 {
                crop_document(doc, x, y, w, h);
            }
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
}

fn crop_document(doc: &mut Document, x: u32, y: u32, w: u32, h: u32) {
    use crate::document::selection::Rect;
    let new_w = w.min(doc.width - x);
    let new_h = h.min(doc.height - y);
    if new_w == 0 || new_h == 0 { return; }
    // Move pixels into the top-left and shrink.
    use crate::render::pixel_buffer::PixelBuffer;
    for layer in &mut doc.layers {
        if let Some(pix) = layer.as_pixel_mut() {
            let mut n = PixelBuffer::new(new_w, new_h);
            for yy in 0..new_h {
                for xx in 0..new_w {
                    n.set_pixel(xx, yy, pix.get_pixel(x + xx, y + yy));
                }
            }
            *pix = n;
        }
        if let Some(mask) = layer.mask_mut() {
            let mut n = PixelBuffer::new(new_w, new_h);
            for yy in 0..new_h {
                for xx in 0..new_w {
                    n.set_pixel(xx, yy, mask.get_pixel(x + xx, y + yy));
                }
            }
            *mask = n;
        }
    }
    doc.width = new_w;
    doc.height = new_h;
    doc.selection.resize(new_w, new_h);
    doc.selection.bounds = Some(Rect { x: 0, y: 0, w: new_w as i32, h: new_h as i32 });
    doc.dirty = true;
}