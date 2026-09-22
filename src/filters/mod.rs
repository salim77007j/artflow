//! Image filters & adjustments.

pub mod adjustments;
pub mod blur;
pub mod distort;
pub mod noise;

use crate::render::pixel_buffer::PixelBuffer;

/// Generic filter function: read from `input` and write into `output`.
pub trait Filter {
    fn apply(&self, input: &PixelBuffer, output: &mut PixelBuffer);
}

/// In-place filter convenience wrapper.
pub fn apply_in_place(filter: &dyn Filter, buf: &mut PixelBuffer) {
    let mut tmp = buf.clone();
    filter.apply(buf, &mut tmp);
    *buf = tmp;
}