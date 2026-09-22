//! Layer model — pixels, mask, blend mode, opacity, transforms.

use crate::color::Rgba;
use crate::render::pixel_buffer::PixelBuffer;
use serde::{Deserialize, Serialize};

pub type LayerId = u32;

/// Blend mode — Photoshop-equivalent list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Add,
    Subtract,
}

impl BlendMode {
    pub fn label(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Multiply => "Multiply",
            BlendMode::Screen => "Screen",
            BlendMode::Overlay => "Overlay",
            BlendMode::Darken => "Darken",
            BlendMode::Lighten => "Lighten",
            BlendMode::ColorDodge => "Color Dodge",
            BlendMode::ColorBurn => "Color Burn",
            BlendMode::SoftLight => "Soft Light",
            BlendMode::HardLight => "Hard Light",
            BlendMode::Difference => "Difference",
            BlendMode::Exclusion => "Exclusion",
            BlendMode::Hue => "Hue",
            BlendMode::Saturation => "Saturation",
            BlendMode::Color => "Color",
            BlendMode::Luminosity => "Luminosity",
            BlendMode::Add => "Add",
            BlendMode::Subtract => "Subtract",
        }
    }

    pub fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal, BlendMode::Multiply, BlendMode::Screen, BlendMode::Overlay,
            BlendMode::Darken, BlendMode::Lighten, BlendMode::ColorDodge, BlendMode::ColorBurn,
            BlendMode::SoftLight, BlendMode::HardLight, BlendMode::Difference, BlendMode::Exclusion,
            BlendMode::Hue, BlendMode::Saturation, BlendMode::Color, BlendMode::Luminosity,
            BlendMode::Add, BlendMode::Subtract,
        ]
    }
}

/// A layer. Could be a raster pixel layer or a shape/text layer (in extensions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend: BlendMode,
    /// Optional layer mask (RGBA where alpha = mask strength).
    pub mask: Option<PixelBuffer>,
    pub kind: LayerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerKind {
    Pixel(PixelBuffer),
    // Extension points: Text(TextLayer), Vector(VectorLayer), etc.
}

impl Layer {
    pub fn new_pixel(name: impl Into<String>, w: u32, h: u32, bg: Rgba) -> Self {
        let mut buf = PixelBuffer::new(w, h);
        buf.clear(bg);
        Self {
            id: 0,
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 1.0,
            blend: BlendMode::Normal,
            mask: None,
            kind: LayerKind::Pixel(buf),
        }
    }

    pub fn as_pixel(&self) -> Option<&PixelBuffer> {
        match &self.kind { LayerKind::Pixel(p) => Some(p), _ => None }
    }
    pub fn as_pixel_mut(&mut self) -> Option<&mut PixelBuffer> {
        match &mut self.kind { LayerKind::Pixel(p) => Some(p), _ => None }
    }

    pub fn width(&self) -> u32 {
        match &self.kind { LayerKind::Pixel(p) => p.width(), _ => 0 }
    }
    pub fn height(&self) -> u32 {
        match &self.kind { LayerKind::Pixel(p) => p.height(), _ => 0 }
    }

    pub fn mask(&self) -> Option<&PixelBuffer> { self.mask.as_ref() }
    pub fn mask_mut(&mut self) -> Option<&mut PixelBuffer> { self.mask.as_mut() }

    pub fn add_mask(&mut self) {
        if self.mask.is_some() { return; }
        let (w, h) = (self.width(), self.height());
        // Default: fully opaque (reveal all).
        let mut m = PixelBuffer::new(w, h);
        m.clear(Rgba::WHITE);
        self.mask = Some(m);
    }

    pub fn remove_mask(&mut self) {
        self.mask = None;
    }
}