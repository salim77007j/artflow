//! The main canvas widget: pan, zoom, draw layers, draw active tool overlay.

use crate::app::ArtFlowApp;
use crate::document::Document;
use crate::render::compositor::flatten_into;
use crate::render::pixel_buffer::PixelBuffer;
use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};

/// Persistent texture cache so we don't re-upload every frame.
#[derive(Clone)]
pub struct CanvasTextures {
    pub composite: Option<TextureHandle>,
    pub layer_previews: Vec<Option<TextureHandle>>,
    pub last_size: (u32, u32),
    pub composite_buf: PixelBuffer,
}

impl Default for CanvasTextures {
    fn default() -> Self {
        Self {
            composite: None,
            layer_previews: Vec::new(),
            last_size: (0, 0),
            composite_buf: PixelBuffer::new(1, 1),
        }
    }
}

/// Render the central canvas.
pub fn show(ui: &mut egui::Ui, doc: &mut Document, app: &mut ArtFlowApp) {
    let available = ui.available_rect_before_wrap();
    let sense = Sense::drag();

    // Compute target zoom (fit-to-view).
    if doc.canvas.fit_to_view {
        let padding = 32.0;
        let sx = (available.width() - padding) / doc.width() as f32;
        let sy = (available.height() - padding) / doc.height() as f32;
        doc.canvas.zoom = sx.min(sy).max(0.05).min(32.0);
        doc.canvas.pan_x = (available.width() - doc.width() as f32 * doc.canvas.zoom) * 0.5;
        doc.canvas.pan_y = (available.height() - doc.height() as f32 * doc.canvas.zoom) * 0.5;
    }

    let response = ui.allocate_rect(available, sense);

    // Checker background.
    let painter = ui.painter_at(available);
    draw_checker(&painter, available);

    let zoom = doc.canvas.zoom;
    let canvas_rect = Rect::from_min_size(
        Pos2::new(available.min.x + doc.canvas.pan_x, available.min.y + doc.canvas.pan_y),
        Vec2::new(doc.width() as f32 * zoom, doc.height() as f32 * zoom),
    );

    // Render composite.
    let textures = ui.ctx().memory(|m| {
        m.data
            .get_temp::<CanvasTextures>(egui::Id::new("artflow_canvas_tex"))
            .unwrap_or_default()
    });
    let mut textures = textures;
    if textures.last_size != (doc.width(), doc.height()) {
        textures.composite = None;
        textures.composite_buf = PixelBuffer::new(doc.width(), doc.height());
        textures.last_size = (doc.width(), doc.height());
    }
    flatten_into(doc, &mut textures.composite_buf);
    let color_image = ColorImage::from_rgba_unmultiplied(
        [textures.composite_buf.width() as usize, textures.composite_buf.height() as usize],
        textures.composite_buf.as_slice(),
    );
    let tex = ui.ctx().load_texture(
        "artflow_composite",
        color_image,
        egui::TextureOptions::LINEAR,
    );
    textures.composite = Some(tex);

    painter.image(
        textures.composite.as_ref().unwrap().id(),
        canvas_rect,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    // Canvas border.
    painter.rect_stroke(canvas_rect, 0.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 80, 120)));

    // Marching ants around selection bounds.
    if let Some(b) = doc.selection.bounds {
        let r = Rect::from_min_size(
            Pos2::new(
                canvas_rect.min.x + b.x as f32 * zoom,
                canvas_rect.min.y + b.y as f32 * zoom,
            ),
            Vec2::new(b.w as f32 * zoom, b.h as f32 * zoom),
        );
        let phase = (ui.input(|i| i.time) * 24.0) as f32;
        let dash = 6.0;
        // Top
        march(&painter, r.left_top(), r.right_top(), phase, dash, Color32::from_rgb(20, 20, 20));
        march(&painter, r.left_bottom(), r.right_bottom(), phase + dash, dash, Color32::from_rgb(255, 255, 255));
        // Sides
        march(&painter, r.left_top(), r.left_bottom(), phase + dash * 2.0, dash, Color32::from_rgb(20, 20, 20));
        march(&painter, r.right_top(), r.right_bottom(), phase + dash * 3.0, dash, Color32::from_rgb(255, 255, 255));
    }

    ui.ctx().memory_mut(|m| {
        m.data.insert_temp(egui::Id::new("artflow_canvas_tex"), textures);
    });

    // Handle pointer interactions.
    handle_pointer(ui, doc, app, &response, canvas_rect);
}

fn draw_checker(painter: &egui::Painter, rect: Rect) {
    let step = 16.0;
    let mut y = rect.min.y;
    let mut row = 0;
    while y < rect.max.y {
        let mut x = rect.min.x;
        let mut col = 0;
        while x < rect.max.x {
            let c = if (row + col) % 2 == 0 {
                Color32::from_rgb(232, 235, 240)
            } else {
                Color32::from_rgb(220, 224, 232)
            };
            let r = Rect::from_min_size(Pos2::new(x, y), Vec2::new(step, step));
            painter.rect_filled(r, 0.0, c);
            x += step;
            col += 1;
        }
        y += step;
        row += 1;
    }
}

fn march(painter: &egui::Painter, a: Pos2, b: Pos2, phase: f32, dash: f32, color: Color32) {
    let dist = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    if dist < 0.001 { return; }
    let dir = Vec2::new((b.x - a.x) / dist, (b.y - a.y) / dist);
    let mut t = -phase;
    while t < dist {
        let t0 = t.max(0.0);
        let t1 = (t + dash * 0.5).min(dist);
        if t1 > t0 {
            let p0 = Pos2::new(a.x + dir.x * t0, a.y + dir.y * t0);
            let p1 = Pos2::new(a.x + dir.x * t1, a.y + dir.y * t1);
            painter.line_segment([p0, p1], Stroke::new(1.0_f32, color));
        }
        t += dash;
    }
}

fn handle_pointer(ui: &mut egui::Ui, doc: &mut Document, app: &mut ArtFlowApp, response: &egui::Response, canvas_rect: Rect) {
    // Pan with middle button / space drag.
    if response.dragged_by(egui::PointerButton::Middle) {
        let d = response.drag_delta();
        doc.canvas.pan_x += d.x;
        doc.canvas.pan_y += d.y;
        doc.canvas.fit_to_view = false;
    }

    // Wheel zoom.
    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll != 0.0 && response.hovered() {
        let old_zoom = doc.canvas.zoom;
        let factor = (1.0 + scroll * 0.0015).max(0.5);
        let new_zoom = (old_zoom * factor).clamp(0.05, 32.0);
        if let Some(pos) = response.hover_pos() {
            let canvas_x = (pos.x - canvas_rect.min.x) / old_zoom;
            let canvas_y = (pos.y - canvas_rect.min.y) / old_zoom;
            doc.canvas.zoom = new_zoom;
            doc.canvas.pan_x = pos.x - canvas_x * new_zoom - canvas_rect.min.x;
            doc.canvas.pan_y = pos.y - canvas_y * new_zoom - canvas_rect.min.y;
            // pan_x/pan_y are offsets from canvas_rect.min; recompute relative to canvas_rect.min.
            doc.canvas.pan_x = pos.x - canvas_x * new_zoom - canvas_rect.min.x;
            doc.canvas.pan_y = pos.y - canvas_y * new_zoom - canvas_rect.min.y;
        } else {
            doc.canvas.zoom = new_zoom;
        }
        doc.canvas.fit_to_view = false;
    }

    // Map pointer position to canvas coords.
    let to_canvas = |pos: Pos2| -> Option<(i32, i32)> {
        if !canvas_rect.contains(pos) { return None; }
        let x = ((pos.x - canvas_rect.min.x) / doc.canvas.zoom) as i32;
        let y = ((pos.y - canvas_rect.min.y) / doc.canvas.zoom) as i32;
        if x < 0 || y < 0 || x >= doc.width() as i32 || y >= doc.height() as i32 { return None; }
        Some((x, y))
    };

    let pos = response.hover_pos();
    let canvas_pt = pos.and_then(to_canvas);

    // Status overlay.
    if let Some(p) = canvas_pt {
        let text = format!("x: {}  y: {}", p.0, p.1);
        let pos_screen = Pos2::new(canvas_rect.min.x + 4.0, canvas_rect.min.y + 4.0);
        ui.painter().text(pos_screen, egui::Align2::LEFT_TOP, text, egui::FontId::monospace(11.0), Color32::from_rgb(60, 80, 120));
    }

    // Hand off to active tool.
    let tool_id = app.active_tool;
    let brush_params = crate::tools::BrushParams {
        size: app.color.brush_size,
        hardness: app.color.brush_hardness,
        opacity: app.color.brush_opacity,
        flow: app.color.brush_flow,
        spacing: app.color.brush_spacing,
    };
    if let Some(layer) = doc.active_layer_mut() {
        if let Some(pix) = layer.as_pixel_mut() {
            if let Some(pt) = canvas_pt {
                let fg = app.color.foreground;
                let bg = app.color.background;
                app.tools.dispatch_pointer(doc, tool_id, pix, pt, response, fg, bg, brush_params);
                // Eyedropper side-effect: sample on click.
                if matches!(tool_id, crate::tools::ToolId::Eyedropper) && response.clicked() {
                    if let Some(c) = crate::tools::eyedropper::pointer(pix, pt, response) {
                        app.color.foreground = c;
                        app.color.hex_input = c.to_hex();
                    }
                }
            }
        }
        // Apply transform overlay
        if let Some(drag) = doc.transform.drag {
            let r = Rect::from_min_max(
                Pos2::new(
                    canvas_rect.min.x + drag.start_x.min(drag.cur_x) * doc.canvas.zoom,
                    canvas_rect.min.y + drag.start_y.min(drag.cur_y) * doc.canvas.zoom,
                ),
                Pos2::new(
                    canvas_rect.min.x + drag.start_x.max(drag.cur_x) * doc.canvas.zoom,
                    canvas_rect.min.y + drag.start_y.max(drag.cur_y) * doc.canvas.zoom,
                ),
            );
            ui.painter().rect_stroke(r, 0.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 130, 240)));
        }
    }
}

// (helpers below kept for future expansion)