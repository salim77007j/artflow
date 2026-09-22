//! Flood fill (paint bucket).

use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;
use crate::document::Document;

pub fn pointer(_doc: &mut Document, layer: &mut PixelBuffer, pt: (i32, i32), color: Rgba) {
    if pt.0 < 0 || pt.1 < 0 || pt.0 >= layer.width() as i32 || pt.1 >= layer.height() as i32 { return; }
    let target = layer.get_pixel(pt.0 as u32, pt.1 as u32);
    let tol = 0.05;
    let mut visited = vec![false; (layer.width() * layer.height()) as usize];
    let mut stack = vec![(pt.0, pt.1)];
    while let Some((cx, cy)) = stack.pop() {
        if cx < 0 || cy < 0 || cx >= layer.width() as i32 || cy >= layer.height() as i32 { continue; }
        let i = (cy as u32 * layer.width() + cx as u32) as usize;
        if visited[i] { continue; }
        visited[i] = true;
        let p = layer.get_pixel(cx as u32, cy as u32);
        if !close_enough(p, target, tol) { continue; }
        let ink = color;
        let dst = p;
        let out_a = ink.a + dst.a * (1.0 - ink.a);
        if out_a <= 0.0 { layer.set_pixel(cx as u32, cy as u32, Rgba::TRANSPARENT); continue; }
        let r = (ink.r * ink.a + dst.r * dst.a * (1.0 - ink.a)) / out_a;
        let g = (ink.g * ink.a + dst.g * dst.a * (1.0 - ink.a)) / out_a;
        let b = (ink.b * ink.a + dst.b * dst.a * (1.0 - ink.a)) / out_a;
        layer.set_pixel(cx as u32, cy as u32, Rgba::new(r, g, b, out_a));
        stack.push((cx + 1, cy));
        stack.push((cx - 1, cy));
        stack.push((cx, cy + 1));
        stack.push((cx, cy - 1));
    }
}

fn close_enough(a: Rgba, b: Rgba, tol: f32) -> bool {
    (a.r - b.r).abs() < tol && (a.g - b.g).abs() < tol && (a.b - b.b).abs() < tol && (a.a - b.a).abs() < tol
}