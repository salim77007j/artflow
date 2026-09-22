//! Transform state — interactive move/scale/rotate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformState {
    /// Optional live drag (in canvas pixel coords).
    pub drag: Option<Drag>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Drag {
    pub start_x: f32,
    pub start_y: f32,
    pub cur_x: f32,
    pub cur_y: f32,
}

impl TransformState {
    pub fn begin(&mut self, x: f32, y: f32) {
        self.drag = Some(Drag { start_x: x, start_y: y, cur_x: x, cur_y: y });
    }
    pub fn update(&mut self, x: f32, y: f32) {
        if let Some(d) = self.drag.as_mut() {
            d.cur_x = x;
            d.cur_y = y;
        }
    }
    pub fn end(&mut self) -> Option<Drag> { self.drag.take() }
}