//! Top-level application state and eframe::App implementation.

use crate::color::ColorState;
use crate::document::{Document, DocumentStore};
use crate::tools::{ToolId, ToolRegistry};
use crate::ui::{panels::PanelState, top_menu, toolbar, workspaces::WorkspaceState};
use eframe::egui::{self, Context, Key, Modifiers};
use log::info;

/// Top-level application state.
///
/// Anything visible at runtime lives here. Sub-systems (documents, tools,
/// panels, history) are owned by the app so that eframe can persist them
/// across frames.
pub struct ArtFlowApp {
    /// All currently-open documents, keyed by an opaque id.
    pub store: DocumentStore,
    /// Currently-active document id (if any).
    pub active_doc: Option<u64>,
    /// Tool registry (brush, eraser, etc.) — central dispatch.
    pub tools: ToolRegistry,
    /// Currently-selected tool id.
    pub active_tool: ToolId,
    /// Color / brush settings panel state.
    pub color: ColorState,
    /// Right-side panel visibility and contents.
    pub panels: PanelState,
    /// Workspace (saved layouts) state.
    pub workspaces: WorkspaceState,
    /// Last-painted brush stamp position — used for spacing interpolation.
    #[allow(dead_code)]
    pub last_stamp: Option<(i32, i32)>,
    /// Pressed key / modifier state that survives one frame (for key chords).
    #[allow(dead_code)]
    pub pending_modifiers: Modifiers,
}

impl ArtFlowApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Register custom fonts here if needed. egui ships with default fonts.
        let _ = cc;

        let mut store = DocumentStore::default();
        // Seed with a welcome document so the canvas isn't blank.
        let welcome = Document::new_welcome(1280, 720);
        let id = store.push(welcome);
        let mut tools = ToolRegistry::new();
        tools.register_defaults();
        Self {
            store,
            active_doc: Some(id),
            tools,
            active_tool: ToolId::Brush,
            color: ColorState::default(),
            panels: PanelState::default(),
            workspaces: WorkspaceState::default(),
            last_stamp: None,
            pending_modifiers: Modifiers::NONE,
        }
    }

    /// Convenience accessor for the active document (mutable).
    pub fn doc_mut(&mut self) -> Option<&mut Document> {
        let id = self.active_doc?;
        self.store.get_mut(id)
    }

    /// Convenience accessor for the active document (immutable).
    pub fn doc(&self) -> Option<&Document> {
        let id = self.active_doc?;
        self.store.get(id)
    }

    /// Switch the active tool. Resets transient tool state.
    pub fn set_tool(&mut self, id: ToolId) {
        self.active_tool = id;
        self.last_stamp = None;
        info!("Active tool → {:?}", id);
    }

    /// Handle a global keyboard shortcut. Returns true if consumed.
    pub fn handle_shortcut(&mut self, ctx: &Context, key: Key, modifiers: Modifiers) -> bool {
        // Ctrl/Cmd based shortcuts.
        let ctrl = modifiers.ctrl || modifiers.command;

        // Tool shortcuts (single keys, no modifiers).
        if !ctrl && !modifiers.alt && !modifiers.shift {
            use ToolId::*;
            let t = match key {
                Key::B => Some(Brush),
                Key::E => Some(Eraser),
                Key::V => Some(Move),
                Key::M => Some(MarqueeRect),
                Key::L => Some(MarqueeEllipse),
                Key::W => Some(MagicWand),
                Key::P => Some(Pencil),
                Key::G => Some(Gradient),
                Key::I => Some(Eyedropper),
                Key::T => Some(Text),
                Key::U => Some(ShapeRect),
                Key::O => Some(ShapeEllipse),
                Key::C => Some(Crop),
                Key::Z if !ctrl => None,
                _ => None,
            };
            if let Some(t) = t {
                self.set_tool(t);
                return true;
            }
        }

        // Undo / Redo.
        if ctrl && matches!(key, Key::Z) && !modifiers.shift && !modifiers.alt {
            if let Some(doc) = self.doc_mut() {
                doc.undo();
                return true;
            }
        }
        if (ctrl && matches!(key, Key::Y))
            || (ctrl && modifiers.shift && matches!(key, Key::Z))
        {
            if let Some(doc) = self.doc_mut() {
                doc.redo();
                return true;
            }
        }

        // Save / Save as.
        if ctrl && matches!(key, Key::S) {
            crate::io::save_dialog(self);
            return true;
        }
        // Open.
        if ctrl && matches!(key, Key::O) {
            crate::io::open_dialog(self);
            return true;
        }
        // New.
        if ctrl && matches!(key, Key::N) {
            crate::io::new_document(self);
            return true;
        }

        // Export flat image (Ctrl+E).
        if ctrl && matches!(key, Key::E) {
            crate::io::export_flat_dialog(self);
            return true;
        }

        // Zoom shortcuts.
        if ctrl && (matches!(key, Key::Equals) || matches!(key, Key::Plus)) {
            if let Some(doc) = self.doc_mut() {
                doc.canvas.zoom = (doc.canvas.zoom * 1.25).min(32.0);
                return true;
            }
        }
        if ctrl && matches!(key, Key::Minus) {
            if let Some(doc) = self.doc_mut() {
                doc.canvas.zoom = (doc.canvas.zoom / 1.25).max(0.05);
                return true;
            }
        }
        if ctrl && matches!(key, Key::Num0) {
            if let Some(doc) = self.doc_mut() {
                doc.canvas.zoom = 1.0;
                return true;
            }
        }

        // Fit to view (Ctrl+1).
        if ctrl && matches!(key, Key::Num1) {
            if let Some(doc) = self.doc_mut() {
                doc.canvas.fit_to_view = true;
                return true;
            }
        }

        // Toggle panels.
        if matches!(key, Key::Tab) && !ctrl {
            self.panels.toggle_right();
            return true;
        }

        // ESC clears selection / cancels tools.
        if matches!(key, Key::Escape) {
            if let Some(doc) = self.doc_mut() {
                doc.selection.clear();
                doc.transform.drag = None;
                return true;
            }
        }

        // Forward to active tool first.
        let tool = self.active_tool;
        if let Some(doc) = self.doc_mut() {
            let tools = &mut self.tools;
            if tools.handle_key(doc, tool, key, modifiers) {
                return true;
            }
        }
        let _ = ctx;
        false
    }
}

impl eframe::App for ArtFlowApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Process global shortcuts.
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed,
                    modifiers,
                    ..
                } = event
                {
                    if *pressed {
                        let k = if let Some(k) = *key { k } else if let Some(k) = *physical_key { k } else { Key::F35 };
                        self.pending_modifiers = *modifiers;
                        self.handle_shortcut(ctx, k, *modifiers);
                    }
                }
            }
        });

        // Top menubar.
        top_menu::show(ctx, self);

        // Bottom status bar (also draws at the very top inside CentralPanel below for simplicity).
        egui::TopBottomPanel::top("toolbar_strip")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(245, 246, 248)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    toolbar::show_secondary_strip(ui, self);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        toolbar::show_topbar_right(ui, self);
                    });
                });
            });

        // Left toolbar.
        egui::SidePanel::left("left_toolbar")
            .resizable(false)
            .exact_width(56.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(252, 253, 255))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 234))),
            )
            .show(ctx, |ui| {
                toolbar::show_left(ui, self);
            });

        // Right panels (Layers / Color / Brush).
        egui::SidePanel::right("right_panels")
            .resizable(true)
            .default_width(280.0)
            .min_width(220.0)
            .max_width(420.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(248, 249, 251))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 234))),
            )
            .show(ctx, |ui| {
                crate::ui::panels::show_right_panels(ui, self);
            });

        // Status bar.
        egui::TopBottomPanel::bottom("statusbar")
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(245, 246, 248))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 234))),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if let Some(doc) = self.doc() {
                        ui.label(format!(
                            "Canvas: {} × {}    Doc: {}    Zoom: {:.0}%",
                            doc.width(),
                            doc.height(),
                            doc.name,
                            doc.canvas.zoom * 100.0
                        ));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(format!("Tool: {:?}", self.active_tool));
                    });
                });
            });

        // Central canvas area.
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(232, 235, 240)))
            .show(ctx, |ui| {
                if let Some(doc) = self.doc_mut() {
                    crate::render::canvas::show(ui, doc, self);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.heading("No document open — press Ctrl+N to start");
                    });
                }
            });

        // Modal dialogs.
        crate::ui::dialogs::show(ctx, self);
    }
}

/// Configure egui look & feel + fonts. Called once at startup.
pub fn configure_egui(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let visuals = &mut style.visuals;

    // Light, modern theme.
    visuals.dark_mode = false;
    visuals.override_text_color = Some(egui::Color32::from_rgb(28, 30, 36));
    visuals.window_fill = egui::Color32::from_rgb(252, 253, 255);
    visuals.panel_fill = egui::Color32::from_rgb(252, 253, 255);
    visuals.extreme_bg_color = egui::Color32::from_rgb(232, 235, 240);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 249, 251);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(225, 228, 234));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 224, 232));
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(238, 242, 250);
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 200, 232));
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(220, 230, 248);
    visuals.widgets.active.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 140, 230));
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);
    visuals.selection.bg_fill = egui::Color32::from_rgb(160, 198, 255);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 130, 240));
    visuals.hyperlink_color = egui::Color32::from_rgb(60, 130, 240);

    // Spacing.
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(8);

    ctx.set_style(style);
}