//! Selection tools: rectangle, ellipse, lasso, magic wand.

use crate::document::{Document, selection::Rect};
use crate::render::pixel_buffer::PixelBuffer;
use crate::tools::ToolState;
use eframe::egui::Response;

pub fn pointer_rect(doc: &mut Document, state: &mut ToolState, pt: (i32, i32), resp: &Response) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() {
        state.drag_cur = Some(pt);
    }
    if resp.drag_released() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let r = Rect::from_points(a.0, a.1, b.0, b.1);
            doc.selection.set_rect(r, 0.0);
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
}

pub fn pointer_ellipse(doc: &mut Document, state: &mut ToolState, pt: (i32, i32), resp: &Response) {
    if resp.drag_started() {
        state.drag_start = Some(pt);
        state.drag_cur = Some(pt);
    }
    if resp.dragged() {
        state.drag_cur = Some(pt);
    }
    if resp.drag_released() {
        if let (Some(a), Some(b)) = (state.drag_start, state.drag_cur) {
            let r = Rect::from_points(a.0, a.1, b.0, b.1);
            doc.selection.set_ellipse(r, 0.0);
        }
        state.drag_start = None;
        state.drag_cur = None;
    }
}

pub fn pointer_lasso(_doc: &mut Document, _state: &mut ToolState, _pt: (i32, i32), _resp: &Response) {
    // Lasso requires free-form path storage — placeholder for now.
}

pub fn pointer_wand(doc: &mut Document, layer: &mut PixelBuffer, pt: (i32, i32), resp: &Response) {
    if resp.clicked() || resp.drag_started() {
        doc.selection.magic_wand(layer, pt.0, pt.1, 32);
    }
}