//! Canvas view (zoom/pan).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasView {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub fit_to_view: bool,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self { zoom: 1.0, pan_x: 0.0, pan_y: 0.0, fit_to_view: true }
    }
}