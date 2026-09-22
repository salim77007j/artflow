//! Text tool — reserved for future expansion. We currently just no-op on click.

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use eframe::egui::Response;

#[allow(dead_code)]
pub fn pointer(_doc: &mut Document, _layer: &mut PixelBuffer, _pt: (i32, i32), _resp: &Response, _fg: Rgba) {
    // Text editing requires an overlay with a text field. We keep the tool
    // selectable so users can map a shortcut to it; actual text rendering
    // uses egui's text widget via a modal dialog in the right panel.
}