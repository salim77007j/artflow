//! Tool system. Each tool is identified by an ID; the registry owns per-tool state.

pub mod brush;
pub mod eraser;
pub mod fill;
pub mod selection;
pub mod shape;
pub mod transform;
pub mod eyedropper;
pub mod text;
pub mod gradient;

use crate::color::Rgba;
use crate::document::Document;
use crate::render::pixel_buffer::PixelBuffer;
use eframe::egui::{Key, Modifiers, Response};

/// Tool identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolId {
    Brush,
    Eraser,
    Pencil,
    Move,
    MarqueeRect,
    MarqueeEllipse,
    Lasso,
    MagicWand,
    Crop,
    Eyedropper,
    Gradient,
    Text,
    ShapeRect,
    ShapeEllipse,
    ShapeLine,
    Fill,
}

impl ToolId {
    pub fn label(&self) -> &'static str {
        match self {
            ToolId::Brush => "Brush",
            ToolId::Eraser => "Eraser",
            ToolId::Pencil => "Pencil",
            ToolId::Move => "Move",
            ToolId::MarqueeRect => "Rect Select",
            ToolId::MarqueeEllipse => "Ellipse Select",
            ToolId::Lasso => "Lasso",
            ToolId::MagicWand => "Magic Wand",
            ToolId::Crop => "Crop",
            ToolId::Eyedropper => "Eyedropper",
            ToolId::Gradient => "Gradient",
            ToolId::Text => "Text",
            ToolId::ShapeRect => "Rectangle",
            ToolId::ShapeEllipse => "Ellipse",
            ToolId::ShapeLine => "Line",
            ToolId::Fill => "Fill",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToolId::Brush => "🖌",
            ToolId::Eraser => "⌫",
            ToolId::Pencil => "✏",
            ToolId::Move => "✥",
            ToolId::MarqueeRect => "▭",
            ToolId::MarqueeEllipse => "◯",
            ToolId::Lasso => "⌒",
            ToolId::MagicWand => "✦",
            ToolId::Crop => "⤢",
            ToolId::Eyedropper => "◉",
            ToolId::Gradient => "▤",
            ToolId::Text => "T",
            ToolId::ShapeRect => "▢",
            ToolId::ShapeEllipse => "⬭",
            ToolId::ShapeLine => "╱",
            ToolId::Fill => "⏍",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            ToolId::Brush => "B",
            ToolId::Eraser => "E",
            ToolId::Pencil => "P",
            ToolId::Move => "V",
            ToolId::MarqueeRect => "M",
            ToolId::MarqueeEllipse => "L",
            ToolId::Lasso => "Shift+L",
            ToolId::MagicWand => "W",
            ToolId::Crop => "C",
            ToolId::Eyedropper => "I",
            ToolId::Gradient => "G",
            ToolId::Text => "T",
            ToolId::ShapeRect => "U",
            ToolId::ShapeEllipse => "O",
            ToolId::ShapeLine => "Shift+U",
            ToolId::Fill => "Shift+F",
        }
    }
}

/// Per-tool runtime state.
#[derive(Debug, Clone, Default)]
pub struct ToolState {
    pub drag_start: Option<(i32, i32)>,
    pub drag_cur: Option<(i32, i32)>,
    pub last_stamp: Option<(i32, i32)>,
}

/// Brush settings snapshot passed at dispatch time.
#[derive(Debug, Clone, Copy)]
pub struct BrushParams {
    pub size: f32,
    pub hardness: f32,
    pub opacity: f32,
    pub flow: f32,
    pub spacing: f32,
}

impl Default for BrushParams {
    fn default() -> Self {
        Self { size: 20.0, hardness: 0.7, opacity: 1.0, flow: 1.0, spacing: 0.1 }
    }
}

/// Registry holding tool state.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    pub state: ToolState,
}

impl ToolRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register_defaults(&mut self) {}

    pub fn dispatch_pointer(
        &mut self,
        doc: &mut Document,
        tool: ToolId,
        layer: &mut PixelBuffer,
        pt: (i32, i32),
        resp: &Response,
        fg: Rgba,
        bg: Rgba,
        params: BrushParams,
    ) {
        match tool {
            ToolId::Brush => brush::pointer(doc, layer, &mut self.state, pt, resp, fg, brush::BrushKind::Round, params.size, params.hardness, params.opacity, params.spacing),
            ToolId::Pencil => brush::pointer(doc, layer, &mut self.state, pt, resp, fg, brush::BrushKind::Square, params.size.max(1.0), 1.0, params.opacity, params.spacing),
            ToolId::Eraser => eraser::pointer(doc, layer, &mut self.state, pt, resp, fg, params.size.max(4.0), params.hardness, params.opacity, params.spacing),
            ToolId::Fill => fill::pointer(doc, layer, pt, fg),
            ToolId::Eyedropper => {
                // Eyedropper sampling is performed in canvas.rs after dispatch (needs app access).
                let _ = eyedropper::pointer(layer, pt, resp);
            }
            ToolId::MarqueeRect => selection::pointer_rect(doc, &mut self.state, pt, resp),
            ToolId::MarqueeEllipse => selection::pointer_ellipse(doc, &mut self.state, pt, resp),
            ToolId::Lasso => selection::pointer_lasso(doc, &mut self.state, pt, resp),
            ToolId::MagicWand => selection::pointer_wand(doc, layer, pt, resp),
            ToolId::Move => transform::pointer_move(doc, &mut self.state, pt, resp),
            ToolId::Crop => transform::pointer_crop(doc, &mut self.state, pt, resp),
            ToolId::ShapeRect => shape::pointer_rect(doc, layer, &mut self.state, pt, resp, fg),
            ToolId::ShapeEllipse => shape::pointer_ellipse(doc, layer, &mut self.state, pt, resp, fg),
            ToolId::ShapeLine => shape::pointer_line(doc, layer, &mut self.state, pt, resp, fg),
            ToolId::Gradient => gradient::pointer(doc, layer, &mut self.state, pt, resp, fg, bg),
            ToolId::Text => text::pointer(doc, layer, pt, resp, fg),
        }
    }

    pub fn handle_key(&mut self, _doc: &mut Document, _tool: ToolId, _key: Key, _modifiers: Modifiers) -> bool {
        false
    }
}