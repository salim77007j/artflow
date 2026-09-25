//! Eyedropper tool — sample a pixel and (here) returns the colour. The
//! canvas widget picks up the returned value via a side-channel pattern.

use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;
use eframe::egui::Response;

pub fn pointer(layer: &mut PixelBuffer, pt: (i32, i32), resp: &Response) -> Option<Rgba> {
    if !resp.clicked() && !resp.drag_started() {
        return None;
    }
    if pt.0 < 0 || pt.1 < 0 || pt.0 >= layer.width() as i32 || pt.1 >= layer.height() as i32 {
        return None;
    }
    Some(layer.get_pixel(pt.0 as u32, pt.1 as u32))
}

/// Read-only variant used by the canvas after a non-mutating eyedropper sample.
pub fn pointer_copy(layer: &PixelBuffer, pt: (i32, i32), resp: &Response) -> Option<Rgba> {
    if !resp.clicked() && !resp.drag_started() {
        return None;
    }
    if pt.0 < 0 || pt.1 < 0 || pt.0 >= layer.width() as i32 || pt.1 >= layer.height() as i32 {
        return None;
    }
    Some(layer.get_pixel(pt.0 as u32, pt.1 as u32))
}