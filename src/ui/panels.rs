//! Right-side panels: Layers, Color Wheel, Brush Settings.

use crate::app::ArtFlowApp;
use crate::color::Rgba;
use crate::document::layer::BlendMode;
use eframe::egui::{self, Color32, RichText, Vec2};

#[derive(Debug, Default)]
pub struct PanelState {
    pub show_right: bool,
    pub show_image_size: bool,
    pub show_canvas_size: bool,
    pub show_blur: bool,
    pub show_sharpen: bool,
    pub show_brightness_contrast: bool,
    pub show_hue_saturation: bool,
    pub show_levels: bool,
    pub show_posterize: bool,
    pub show_threshold: bool,
    pub show_noise: bool,
    pub show_swirl: bool,
    pub show_settings: bool,
    pub show_text_editor: bool,
    // Slider values
    pub blur_radius: f32,
    pub sharpen_amount: f32,
    pub bc_brightness: f32,
    pub bc_contrast: f32,
    pub hue_shift: f32,
    pub sat_mult: f32,
    pub val_mult: f32,
    pub levels_black: f32,
    pub levels_gamma: f32,
    pub levels_white: f32,
    pub posterize_levels: u32,
    pub threshold_value: f32,
    pub noise_amount: f32,
    pub swirl_strength: f32,
    // Modal values
    pub modal_new_w: u32,
    pub modal_new_h: u32,
}

impl PanelState {
    pub fn toggle_right(&mut self) { self.show_right = !self.show_right; }
}

pub fn show_right_panels(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    if !app.panels.show_right {
        ui.centered_and_justified(|ui| { ui.label("Panels hidden — press Tab to show"); });
        return;
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        layers_panel(ui, app);
        ui.add_space(6.0);
        color_panel(ui, app);
        ui.add_space(6.0);
        brush_panel(ui, app);
    });
}

fn layers_panel(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    egui::CollapsingHeader::new(RichText::new("Layers").strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("＋").on_hover_text("New layer").clicked() {
                    if let Some(d) = app.doc_mut() {
                        d.add_pixel_layer(format!("Layer {}", d.layer_count()));
                    }
                }
                if ui.button("⧉").on_hover_text("Duplicate").clicked() {
                    if let Some(d) = app.doc_mut() {
                        if let Some(id) = d.active_layer { d.duplicate_layer(id); }
                    }
                }
                if ui.button("🗑").on_hover_text("Delete").clicked() {
                    if let Some(d) = app.doc_mut() {
                        if let Some(id) = d.active_layer { d.delete_layer(id); }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("↧").on_hover_text("Move down").clicked() {
                        if let Some(d) = app.doc_mut() { if let Some(id) = d.active_layer { d.move_layer(id, 1); } }
                    }
                    if ui.button("↥").on_hover_text("Move up").clicked() {
                        if let Some(d) = app.doc_mut() { if let Some(id) = d.active_layer { d.move_layer(id, -1); } }
                    }
                });
            });

            // Header row (Normal / Opacity).
            ui.horizontal(|ui| {
                let blend_label: BlendMode = app.doc()
                    .and_then(|d| d.active_layer)
                    .and_then(|id| {
                        let doc = app.doc();
                        doc.and_then(|d2| d2.layer(id)).map(|l| l.blend)
                    })
                    .unwrap_or_default();
                egui::ComboBox::from_label("")
                    .selected_text(blend_label.label())
                    .show_ui(ui, |ui| {
                        for m in BlendMode::all() {
                            if ui.selectable_label(blend_label == *m, m.label()).clicked() {
                                if let Some(d) = app.doc_mut() {
                                    if let Some(id) = d.active_layer { d.set_layer_blend(id, *m); }
                                }
                            }
                        }
                    });
                ui.label("Opacity");
                let mut op: f32 = app.doc()
                    .and_then(|d| d.active_layer)
                    .and_then(|id| {
                        let doc = app.doc();
                        doc.and_then(|d2| d2.layer(id)).map(|l| l.opacity * 100.0)
                    })
                    .unwrap_or(100.0);
                if ui.add(egui::Slider::new(&mut op, 0.0..=100.0_f32).show_value(true)).changed() {
                    if let Some(d) = app.doc_mut() {
                        if let Some(id) = d.active_layer { d.set_layer_opacity(id, op / 100.0); }
                    }
                }
            });

            // Layer rows.
            let doc = app.doc();
            if let Some(d) = doc {
                let active = d.active_layer;
                let layers = d.layers.clone();
                for layer in layers.iter().rev() {
                    let is_active = active == Some(layer.id);
                    let frame = egui::Frame::none()
                        .fill(if is_active { Color32::from_rgb(232, 240, 252) } else { Color32::TRANSPARENT })
                        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(225, 228, 234)));
                    egui::Frame::show(frame, ui, |ui| {
                        ui.horizontal(|ui| {
                            // Visibility toggle.
                            let mut vis = layer.visible;
                            if ui.checkbox(&mut vis, "").changed() {
                                if let Some(d) = app.doc_mut() {
                                    d.set_layer_visible(layer.id, vis);
                                }
                            }
                            // Thumbnail.
                            if let Some(pix) = layer.as_pixel() {
                                let thumb = thumbnail(ui.ctx(), pix);
                                ui.image(&thumb);
                            }
                            ui.vertical(|ui| {
                                ui.label(&layer.name);
                                ui.label(format!("Opacity {:.0}%", layer.opacity * 100.0));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(if layer.locked { "🔒" } else { "🔓" }).clicked() {
                                    if let Some(d) = app.doc_mut() {
                                        if let Some(l) = d.layer_mut(layer.id) { l.locked = !l.locked; }
                                    }
                                }
                            });
                        });
                    });
                    // Click handler outside the frame is tricky — we just rely on the row's frame covering it.
                    let resp = ui.allocate_response(Vec2::new(ui.available_width(), 32.0), egui::Sense::click());
                    if resp.clicked() {
                        if let Some(d) = app.doc_mut() {
                            d.active_layer = Some(layer.id);
                        }
                    }
                }
            }
        });
}

fn color_panel(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    egui::CollapsingHeader::new(RichText::new("Color").strong())
        .default_open(true)
        .show(ui, |ui| {
            // Color wheel area.
            let desired = egui::vec2(ui.available_width() - 16.0, 200.0);
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::drag());
            let painter = ui.painter_at(rect);
            draw_color_wheel(&painter, rect);
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if rect.contains(pos) {
                    if ui.input(|i| i.pointer.primary_down()) {
                        let c = sample_color_wheel(pos, rect);
                        app.color.foreground = c;
                        app.color.hex_input = c.to_hex();
                    }
                }
            }
            // Hex input.
            ui.horizontal(|ui| {
                ui.label("HEX");
                let resp = ui.text_edit_singleline(&mut app.color.hex_input);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    app.color.set_fg_hex(&app.color.hex_input);
                }
            });
            // Swatches.
            ui.add_space(4.0);
            ui.label("Swatches");
            let mut i = 0;
            while i < app.color.swatches.len() {
                ui.horizontal(|ui| {
                    for j in 0..8 {
                        if i + j >= app.color.swatches.len() { break; }
                        let idx = i + j;
                        let c = app.color.swatches[idx];
                        let (r, g, b, _) = c.to_rgba8();
                        let rect_size = 22.0;
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(rect_size, rect_size), egui::Sense::click());
                        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(r, g, b));
                        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 60)));
                        if ui.input(|i| i.pointer.primary_clicked()) && rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                            app.color.foreground = c;
                        }
                    }
                });
                i += 8;
            }
        });
}

fn brush_panel(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    egui::CollapsingHeader::new(RichText::new("Brush Settings").strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| { ui.label("Brush Size"); ui.add(egui::DragValue::new(&mut app.color.brush_size).range(0.5..=512.0)); });
            ui.horizontal(|ui| { ui.label("Opacity"); ui.add(egui::Slider::new(&mut app.color.brush_opacity, 0.0..=1.0).show_value(true)); });
            ui.horizontal(|ui| { ui.label("Flow"); ui.add(egui::Slider::new(&mut app.color.brush_flow, 0.0..=1.0).show_value(true)); });
            ui.horizontal(|ui| { ui.label("Hardness"); ui.add(egui::Slider::new(&mut app.color.brush_hardness, 0.0..=1.0).show_value(true)); });
            ui.horizontal(|ui| { ui.label("Spacing"); ui.add(egui::Slider::new(&mut app.color.brush_spacing, 0.0..=1.0).show_value(true)); });
            ui.add_space(4.0);
            // Brush preview.
            let rect = ui.allocate_exact_size(egui::vec2(ui.available_width() - 16.0, 60.0), egui::Sense::hover()).0;
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, Color32::WHITE);
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0_f32, Color32::from_rgb(225, 228, 234)));
            let cx = rect.min.x + rect.width() * 0.5_f32;
            let cy = rect.min.y + rect.height() * 0.5_f32;
            let (fg_r, fg_g, fg_b, _) = app.color.foreground.to_rgba8();
            let col = Color32::from_rgb(fg_r, fg_g, fg_b);
            let stamp_r = (app.color.brush_size.min(rect.width() * 0.45_f32) * 0.5_f32).max(2.0_f32);
            // Approximate soft stamp.
            let steps: u32 = 64;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let rad = stamp_r * t;
                let a = (1.0_f32 - t).powf((1.0_f32 - app.color.brush_hardness).max(0.01_f32) * 2.0_f32) * 255.0_f32;
                painter.circle_filled(
                    egui::pos2(cx, cy),
                    rad,
                    Color32::from_rgba_unmultiplied(fg_r, fg_g, fg_b, a as u8),
                );
            }
            let _ = col;
        });
}

fn draw_color_wheel(painter: &egui::Painter, rect: egui::Rect) {
    let center = rect.center();
    let r = rect.width().min(rect.height()) * 0.5 - 4.0_f32;
    let steps = 96;
    for i in 0..steps {
        let a0 = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let a1 = ((i + 1) as f32 / steps as f32) * std::f32::consts::TAU;
        let c0 = Rgba::from_hsl(i as f32 / steps as f32, 1.0_f32, 0.5_f32);
        let c1 = Rgba::from_hsl((i + 1) as f32 / steps as f32, 1.0_f32, 0.5_f32);
        let p0 = center + egui::vec2(a0.cos() * r, a0.sin() * r);
        let p1 = center + egui::vec2(a0.cos() * (r * 0.3_f32), a0.sin() * (r * 0.3_f32));
        let p2 = center + egui::vec2(a1.cos() * (r * 0.3_f32), a1.sin() * (r * 0.3_f32));
        let p3 = center + egui::vec2(a1.cos() * r, a1.sin() * r);
        let (r0, g0, b0, _) = c0.to_rgba8();
        let (r1, g1, b1, _) = c1.to_rgba8();
        let mut mesh = egui::Mesh::default();
        let i0: u32 = mesh.vertices.len() as u32; mesh.colored_vertex(p0, Color32::from_rgb(r0, g0, b0));
        let i1: u32 = mesh.vertices.len() as u32; mesh.colored_vertex(p1, Color32::WHITE);
        let i2: u32 = mesh.vertices.len() as u32; mesh.colored_vertex(p2, Color32::WHITE);
        let i3: u32 = mesh.vertices.len() as u32; mesh.colored_vertex(p3, Color32::from_rgb(r1, g1, b1));
        mesh.add_triangle(i0, i1, i2);
        mesh.add_triangle(i0, i2, i3);
        painter.add(egui::Shape::mesh(mesh));
    }
    painter.circle_stroke(center, r, egui::Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 60)));
}

fn sample_color_wheel(pos: egui::Pos2, rect: egui::Rect) -> Rgba {
    let center = rect.center();
    let dx = pos.x - center.x;
    let dy = pos.y - center.y;
    let r = rect.width().min(rect.height()) * 0.5 - 4.0;
    let dist = (dx * dx + dy * dy).sqrt();
    let theta = dy.atan2(dx);
    let h = (theta / std::f32::consts::TAU + 1.0).rem_euclid(1.0);
    let s = (dist / r).clamp(0.0, 1.0);
    let l = 0.5;
    Rgba::from_hsl(h, s, l)
}

fn thumbnail(ctx: &egui::Context, pix: &crate::render::pixel_buffer::PixelBuffer) -> egui::TextureHandle {
    let w = pix.width().min(64) as usize;
    let h = pix.height().min(64) as usize;
    let mut small = vec![0u8; w * h * 4];
    let sx = pix.width() as f32 / w as f32;
    let sy = pix.height() as f32 / h as f32;
    for y in 0..h {
        for x in 0..w {
            let src_x = (x as f32 * sx) as u32;
            let src_y = (y as f32 * sy) as u32;
            let p = pix.get_pixel(src_x.min(pix.width() - 1), src_y.min(pix.height() - 1));
            let (r, g, b, a) = p.to_rgba8();
            let idx = (y * w + x) * 4;
            small[idx] = r; small[idx + 1] = g; small[idx + 2] = b; small[idx + 3] = a;
        }
    }
    let img = egui::ColorImage::from_rgba_unmultiplied([w, h], &small);
    ctx.load_texture("layer_thumb", img, egui::TextureOptions::LINEAR)
}