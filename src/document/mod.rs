//! Document / Layer / History. This is the heart of the editor's data model.

pub mod blend;
pub mod canvas;
pub mod history;
pub mod layer;
pub mod selection;
pub mod transform;

use crate::color::Rgba;
use crate::document::canvas::CanvasView;
use crate::document::history::{HistoryStack, Operation};
use crate::document::layer::{Layer, LayerId};
use crate::document::selection::Selection;
use crate::document::transform::TransformState;
use serde::{Deserialize, Serialize};

pub use layer::BlendMode;

/// Top-level editable document (one open file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Display name (file name without extension).
    pub name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Background fill used when creating a new doc.
    pub background: Rgba,
    /// Ordered layers — index 0 is the bottom.
    pub layers: Vec<Layer>,
    /// Currently-selected layer id (if any).
    pub active_layer: Option<LayerId>,
    /// Canvas view (zoom / pan).
    pub canvas: CanvasView,
    /// Selection state (marching ants, mask).
    pub selection: Selection,
    /// Active transform.
    pub transform: TransformState,
    /// Undo / redo.
    pub history: HistoryStack,
    /// Dirty flag.
    pub dirty: bool,
    /// Path the doc was loaded from, if any.
    pub path: Option<std::path::PathBuf>,
}

impl Document {
    pub fn new(name: impl Into<String>, width: u32, height: u32, bg: Rgba) -> Self {
        let mut doc = Self {
            name: name.into(),
            width,
            height,
            background: bg,
            layers: Vec::new(),
            active_layer: None,
            canvas: CanvasView::default(),
            selection: Selection::new(width, height),
            transform: TransformState::default(),
            history: HistoryStack::new(),
            dirty: false,
            path: None,
        };
        // Seed with one base layer.
        let id = doc.layers.len() as LayerId;
        doc.layers.push(Layer::new_pixel("Background", width, height, bg));
        doc.active_layer = Some(id);
        doc
    }

    /// Create a friendly welcome document with a gradient background.
    pub fn new_welcome(width: u32, height: u32) -> Self {
        let mut doc = Self::new("Untitled", width, height, Rgba::WHITE);
        // Paint a soft diagonal gradient onto the background layer.
        if let Some(l) = doc.layers.get_mut(0) {
            if let Some(pixel) = l.as_pixel_mut() {
                let w = pixel.width();
                let h = pixel.height();
                for y in 0..h {
                    for x in 0..w {
                        let t = (x + y) as f32 / (w + h) as f32;
                        let c = Rgba::new(0.94 - 0.04 * t, 0.95 - 0.02 * t, 0.97, 1.0);
                        pixel.set_pixel(x, y, c);
                    }
                }
            }
        }
        doc
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }

    /// Number of layers.
    pub fn layer_count(&self) -> usize { self.layers.len() }

    /// Active layer, mutable.
    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        let id = self.active_layer?;
        self.layers.iter_mut().find(|l| l.id == id)
    }
    /// Active layer, immutable.
    pub fn active_layer(&self) -> Option<&Layer> {
        let id = self.active_layer?;
        self.layers.iter().find(|l| l.id == id)
    }

    /// Find a layer by id (mutable).
    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Add a new layer above the active one (or on top if none active).
    pub fn add_layer(&mut self, name: impl Into<String>, layer: Layer) -> LayerId {
        let new_id = self.layers.len() as LayerId;
        let insert_at = match self.active_layer {
            Some(active) => self.layers.iter().position(|l| l.id == active).map(|i| i + 1).unwrap_or(self.layers.len()),
            None => self.layers.len(),
        };
        let mut layer = layer;
        layer.id = new_id;
        let name = name.into();
        layer.name = name;
        self.layers.insert(insert_at, layer);
        self.active_layer = Some(new_id);
        self.history.push(Operation::LayerAdded { id: new_id, name: self.layers[insert_at].name.clone() });
        self.dirty = true;
        new_id
    }

    pub fn add_pixel_layer(&mut self, name: impl Into<String>) -> LayerId {
        self.add_layer(name, Layer::new_pixel("Layer", self.width, self.height, Rgba::TRANSPARENT))
    }

    pub fn delete_layer(&mut self, id: LayerId) {
        if let Some(idx) = self.layers.iter().position(|l| l.id == id) {
            let layer = self.layers.remove(idx);
            self.history.push(Operation::LayerRemoved { id, snapshot: layer });
            if self.active_layer == Some(id) {
                self.active_layer = self.layers.get(idx).map(|l| l.id)
                    .or_else(|| self.layers.last().map(|l| l.id));
            }
            self.dirty = true;
        }
    }

    pub fn duplicate_layer(&mut self, id: LayerId) {
        if let Some(idx) = self.layers.iter().position(|l| l.id == id) {
            let clone = self.layers[idx].clone();
            let new_id = self.layers.len() as LayerId;
            let mut dup = clone;
            dup.id = new_id;
            dup.name = format!("{} copy", dup.name);
            self.layers.insert(idx + 1, dup);
            self.active_layer = Some(new_id);
            self.history.push(Operation::LayerAdded { id: new_id, name: self.layers[idx + 1].name.clone() });
            self.dirty = true;
        }
    }

    pub fn move_layer(&mut self, id: LayerId, dir: i32) {
        if let Some(idx) = self.layers.iter().position(|l| l.id == id) {
            let new_idx = (idx as i32 + dir).clamp(0, self.layers.len() as i32 - 1) as usize;
            if new_idx != idx {
                let layer = self.layers.remove(idx);
                self.layers.insert(new_idx, layer);
                self.dirty = true;
            }
        }
    }

    pub fn set_layer_visible(&mut self, id: LayerId, v: bool) {
        if let Some(l) = self.layer_mut(id) { l.visible = v; self.dirty = true; }
    }
    pub fn set_layer_opacity(&mut self, id: LayerId, o: f32) {
        if let Some(l) = self.layer_mut(id) { l.opacity = o.clamp(0.0, 1.0); self.dirty = true; }
    }
    pub fn set_layer_blend(&mut self, id: LayerId, m: BlendMode) {
        if let Some(l) = self.layer_mut(id) { l.blend = m; self.dirty = true; }
    }

    /// Flatten all visible layers into the bottom layer (Photoshop "Flatten Image").
    pub fn flatten(&mut self) {
        if self.layers.len() < 2 { return; }
        // Compose everything onto the bottom layer.
        let bottom_id = self.layers[0].id;
        // Copy a snapshot of current state to render the merge.
        let composed = crate::render::compositor::flatten_visible(self);
        // Replace the bottom layer.
        if let Some(bottom) = self.layer_mut(bottom_id) {
            if let Some(pix) = bottom.as_pixel_mut() {
                if let Some(comp) = composed {
                    *pix = comp;
                }
            }
        }
        // Remove all layers above bottom.
        let removed: Vec<Layer> = self.layers.drain(1..).collect();
        self.history.push(Operation::LayersFlattened { count: removed.len() + 1 });
        self.active_layer = Some(bottom_id);
        self.dirty = true;
    }

    /// Resize canvas (preserves layer content where it overlaps; pads with background).
    pub fn resize_canvas(&mut self, new_w: u32, new_h: u32) {
        let old_w = self.width;
        let old_h = self.height;
        if new_w == old_w && new_h == old_h { return; }
        for layer in &mut self.layers {
            if let Some(pix) = layer.as_pixel_mut() {
                pix.resize_canvas(new_w, new_h, self.background);
            }
            if let Some(mask) = layer.mask_mut() {
                mask.resize_canvas(new_w, new_h, Rgba::TRANSPARENT);
            }
        }
        self.width = new_w;
        self.height = new_h;
        self.selection.resize(new_w, new_h);
        self.dirty = true;
        self.history.push(Operation::CanvasResized {
            old: (old_w, old_h),
            new: (new_w, new_h),
        });
    }

    /// Undo last operation.
    pub fn undo(&mut self) {
        let Some(op) = self.history.undo() else { return };
        self.apply_inverse(&op);
        self.dirty = true;
    }
    /// Redo last undone operation.
    pub fn redo(&mut self) {
        let Some(op) = self.history.redo() else { return };
        self.apply_forward(&op);
        self.dirty = true;
    }

    fn apply_forward(&mut self, op: &Operation) {
        match op {
            Operation::LayerAdded { id, .. } => {
                self.active_layer = Some(*id);
            }
            Operation::LayerRemoved { .. } => {}
            Operation::LayersFlattened { .. } => {}
            Operation::CanvasResized { new, .. } => {
                self.resize_canvas(new.0, new.1);
            }
            Operation::PixelsChanged { .. } => {
                // Snapshot restoration handled implicitly because we keep one snapshot.
            }
        }
    }

    fn apply_inverse(&mut self, op: &Operation) {
        match op {
            Operation::LayerAdded { id, .. } => {
                self.delete_layer(*id);
            }
            Operation::LayerRemoved { id, snapshot } => {
                if let Some(idx) = self.layers.iter().position(|l| l.id == *id) {
                    let insert_at = idx.min(self.layers.len());
                    self.layers.insert(insert_at, snapshot.clone());
                } else {
                    self.layers.push(snapshot.clone());
                }
            }
            Operation::LayersFlattened { count } => {
                // Simple approximation — re-seed with empty layers if needed.
                let needed = self.layers.len() < *count;
                if needed {
                    while self.layers.len() < *count {
                        let id = self.layers.len() as LayerId;
                        let mut l = Layer::new_pixel("Recovered", self.width, self.height, Rgba::TRANSPARENT);
                        l.id = id;
                        self.layers.push(l);
                    }
                }
            }
            Operation::CanvasResized { old, .. } => {
                self.resize_canvas(old.0, old.1);
            }
            Operation::PixelsChanged { .. } => {}
        }
    }
}

/// Stores all open documents in the application.
#[derive(Default)]
pub struct DocumentStore {
    docs: Vec<Document>,
    next_id: u64,
}

impl DocumentStore {
    pub fn push(&mut self, doc: Document) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.docs.push(doc);
        id
    }
    pub fn get(&self, _id: u64) -> Option<&Document> { self.docs.first() }
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Document> {
        // We're using index-based identity for now.
        self.docs.get_mut(id as usize)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Document> { self.docs.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Document> { self.docs.iter_mut() }
    pub fn close(&mut self, id: u64) { self.docs.remove(id as usize); }
    pub fn count(&self) -> usize { self.docs.len() }
}