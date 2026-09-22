//! Pixel-level blending math.
//!
//! All operations are done in straight-alpha, non-premultiplied space. Compositing
//! happens via Porter-Duff "over" at the end.

use crate::color::Rgba;
use crate::document::layer::BlendMode;

/// Compute blended output = blend(src, dst) using the given mode.
pub fn blend(src: Rgba, dst: Rgba, mode: BlendMode) -> Rgba {
    let (sr, sg, sb) = (src.r, src.g, src.b);
    let (dr, dg, db) = (dst.r, dst.g, dst.b);
    let out = match mode {
        BlendMode::Normal => (sr, sg, sb),
        BlendMode::Multiply => (sr * dr, sg * dg, sb * db),
        BlendMode::Screen => (1.0 - (1.0 - sr) * (1.0 - dr),
                              1.0 - (1.0 - sg) * (1.0 - dg),
                              1.0 - (1.0 - sb) * (1.0 - db)),
        BlendMode::Overlay => {
            (overlay_channel(dr, sr), overlay_channel(dg, sg), overlay_channel(db, sb))
        }
        BlendMode::Darken => (sr.min(dr), sg.min(dg), sb.min(db)),
        BlendMode::Lighten => (sr.max(dr), sg.max(dg), sb.max(db)),
        BlendMode::ColorDodge => {
            (dodge(dr, sr), dodge(dg, sg), dodge(db, sb))
        }
        BlendMode::ColorBurn => {
            (burn(dr, sr), burn(dg, sg), burn(db, sb))
        }
        BlendMode::SoftLight => {
            (soft_light(dr, sr), soft_light(dg, sg), soft_light(db, sb))
        }
        BlendMode::HardLight => {
            (overlay_channel(sr, dr), overlay_channel(sg, dg), overlay_channel(sb, db))
        }
        BlendMode::Difference => ((sr - dr).abs(), (sg - dg).abs(), (sb - db).abs()),
        BlendMode::Exclusion => (sr + dr - 2.0 * sr * dr,
                                  sg + dg - 2.0 * sg * dg,
                                  sb + db - 2.0 * sb * db),
        BlendMode::Hue => {
            // Hue: dst's luma with src's hue/saturation.
            let (h, s, _) = src.to_hsl();
            let (_, _, l) = dst.to_hsl();
            let c = Rgba::from_hsl(h, s, l);
            (c.r, c.g, c.b)
        }
        BlendMode::Saturation => {
            let (_, s, _) = src.to_hsl();
            let (h, _, l) = dst.to_hsl();
            let c = Rgba::from_hsl(h, s, l);
            (c.r, c.g, c.b)
        }
        BlendMode::Color => {
            let (h, s, _) = src.to_hsl();
            let (_, _, l) = dst.to_hsl();
            let c = Rgba::from_hsl(h, s, l);
            (c.r, c.g, c.b)
        }
        BlendMode::Luminosity => {
            let (h, s, _) = dst.to_hsl();
            let (_, _, l) = src.to_hsl();
            let c = Rgba::from_hsl(h, s, l);
            (c.r, c.g, c.b)
        }
        BlendMode::Add => ((sr + dr).min(1.0), (sg + dg).min(1.0), (sb + db).min(1.0)),
        BlendMode::Subtract => ((sr - dr).max(0.0), (sg - dg).max(0.0), (sb - db).max(0.0)),
    };
    Rgba::new(out.0.clamp(0.0, 1.0), out.1.clamp(0.0, 1.0), out.2.clamp(0.0, 1.0), src.a)
}

fn overlay_channel(dst: f32, src: f32) -> f32 {
    if dst < 0.5 {
        2.0 * src * dst
    } else {
        1.0 - 2.0 * (1.0 - src) * (1.0 - dst)
    }
}

fn dodge(dst: f32, src: f32) -> f32 {
    if src >= 1.0 { 1.0 } else { (dst / (1.0 - src)).min(1.0) }
}

fn burn(dst: f32, src: f32) -> f32 {
    if src <= 0.0 { 0.0 } else { 1.0 - ((1.0 - dst) / src).min(1.0).max(0.0) }
}

fn soft_light(dst: f32, src: f32) -> f32 {
    if src < 0.5 {
        dst - (1.0 - 2.0 * src) * dst * (1.0 - dst)
    } else {
        let d = if dst <= 0.25 {
            ((16.0 * dst - 12.0) * dst + 4.0) * dst
        } else {
            dst.sqrt()
        };
        dst + (2.0 * src - 1.0) * (d - dst)
    }
}

/// Composite `src` over `dst` using straight alpha + Porter-Duff "over".
pub fn composite_over(src: Rgba, dst: Rgba) -> Rgba {
    let sa = src.a;
    let da = dst.a;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return Rgba::TRANSPARENT;
    }
    let r = (src.r * sa + dst.r * da * (1.0 - sa)) / out_a;
    let g = (src.g * sa + dst.g * da * (1.0 - sa)) / out_a;
    let b = (src.b * sa + dst.b * da * (1.0 - sa)) / out_a;
    Rgba::new(r, g, b, out_a)
}