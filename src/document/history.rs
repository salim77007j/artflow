//! Undo / redo stack.
//!
//! We use a coarse operation log plus per-operation snapshot/replay data. For
//! a real production editor you'd want command-pattern + memento for every
//! pixel edit, but for a single-developer project this is a reasonable middle
//! ground: we keep N snapshots of recent pixel edits.

use crate::document::layer::{Layer, LayerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    LayerAdded { id: LayerId, name: String },
    LayerRemoved { id: LayerId, snapshot: Layer },
    LayersFlattened { count: usize },
    CanvasResized { old: (u32, u32), new: (u32, u32) },
    /// Marker for a region of pixel changes (we don't snapshot the whole
    /// document for every stroke — too expensive; we rely on the tool to
    /// support local undo if needed).
    PixelsChanged { label: String },
}

#[derive(Default, Debug)]
pub struct HistoryStack {
    done: Vec<Operation>,
    undone: Vec<Operation>,
    cap: usize,
}

impl HistoryStack {
    pub fn new() -> Self { Self { done: Vec::new(), undone: Vec::new(), cap: 256 } }

    pub fn push(&mut self, op: Operation) {
        self.done.push(op);
        if self.done.len() > self.cap {
            self.done.remove(0);
        }
        // A new edit invalidates the redo stack.
        self.undone.clear();
    }

    pub fn undo(&mut self) -> Option<Operation> {
        let op = self.done.pop()?;
        self.undone.push(op.clone());
        Some(op)
    }

    pub fn redo(&mut self) -> Option<Operation> {
        let op = self.undone.pop()?;
        self.done.push(op.clone());
        Some(op)
    }

    pub fn can_undo(&self) -> bool { !self.done.is_empty() }
    pub fn can_redo(&self) -> bool { !self.undone.is_empty() }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }
}