//! Composites all visible layers of a document into a single pixel buffer.

use crate::color::Rgba;
use crate::document::blend::{blend, composite_over};
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;

/// Compose every visible layer bottom-up into a new PixelBuffer.
pub fn flatten_visible(doc: &Document) -> Option<PixelBuffer> {
    let mut acc = PixelBuffer::new(doc.width, doc.height);
    acc.clear(Rgba::TRANSPARENT);
    for layer in &doc.layers {
        if !layer.visible { continue; }
        let Some(pix) = layer.as_pixel() else { continue; };
        let w = doc.width.min(pix.width());
        let h = doc.height.min(pix.height());
        for y in 0..h {
            for x in 0..w {
                let src = pix.get_pixel(x, y);
                // Apply mask if present.
                let masked = if let Some(mask) = layer.mask() {
                    let m = mask.get_pixel(x, y);
                    Rgba::new(src.r, src.g, src.b, src.a * m.a)
                } else {
                    src
                };
                let opacity = layer.opacity;
                let mut tinted = Rgba::new(masked.r, masked.g, masked.b, masked.a * opacity);
                if tinted.a <= 0.0 { continue; }
                let dst = acc.get_pixel(x, y);
                let blended = blend(tinted, dst, layer.blend);
                let final_color = composite_over(blended, dst);
                acc.set_pixel(x, y, final_color);
            }
        }
    }
    Some(acc)
}

/// Compose a single layer preview at given bounds (used by thumbnails).
pub fn flatten_into(doc: &Document, target: &mut PixelBuffer) {
    target.clear(Rgba::TRANSPARENT);
    let w = doc.width.min(target.width());
    let h = doc.height.min(target.height());
    for layer in &doc.layers {
        if !layer.visible { continue; }
        let Some(pix) = layer.as_pixel() else { continue; };
        for y in 0..h {
            for x in 0..w {
                let src = pix.get_pixel(x, y);
                let masked = if let Some(mask) = layer.mask() {
                    let m = mask.get_pixel(x, y);
                    Rgba::new(src.r, src.g, src.b, src.a * m.a)
                } else {
                    src
                };
                let tinted = Rgba::new(masked.r, masked.g, masked.b, masked.a * layer.opacity);
                let dst = target.get_pixel(x, y);
                let blended = blend(tinted, dst, layer.blend);
                let final_color = composite_over(blended, dst);
                target.set_pixel(x, y, final_color);
            }
        }
    }
}