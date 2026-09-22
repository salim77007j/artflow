//! Top menubar with File / Edit / Image / Layer / Filter / View / Window menus.

use crate::app::ArtFlowApp;
use eframe::egui::{self, Button};

pub fn show(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::TopBottomPanel::top("menubar")
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(252, 253, 255))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(225, 228, 234))),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("✦ ArtFlow")
                        .strong()
                        .color(egui::Color32::from_rgb(60, 130, 240))
                        .size(15.0),
                );
                ui.add_space(12.0);

                file_menu(ui, app);
                edit_menu(ui, app);
                image_menu(ui, app);
                layer_menu(ui, app);
                filter_menu(ui, app);
                view_menu(ui, app);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if ui.button(egui::RichText::new("⚙").size(14.0)).clicked() {
                        app.panels.show_settings = true;
                    }
                    if let Some(doc) = app.doc() {
                        if doc.dirty {
                            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(220, 80, 80)));
                        }
                    }
                });
            });
        });
}

fn file_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("File", |ui| {
        if ui.button("New (Ctrl+N)").clicked() {
            crate::io::new_document(app);
            ui.close_menu();
        }
        if ui.button("Open… (Ctrl+O)").clicked() {
            crate::io::open_dialog(app);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Save (Ctrl+S)").clicked() {
            crate::io::save_dialog(app);
            ui.close_menu();
        }
        if ui.button("Export Flat… (Ctrl+E)").clicked() {
            crate::io::export_flat_dialog(app);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Quit").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn edit_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("Edit", |ui| {
        let undo = app.doc().map(|d| d.history.can_undo()).unwrap_or(false);
        let redo = app.doc().map(|d| d.history.can_redo()).unwrap_or(false);
        if ui.add_enabled(undo, egui::Button::new("Undo (Ctrl+Z)")).clicked() {
            if let Some(d) = app.doc_mut() { d.undo(); }
            ui.close_menu();
        }
        if ui.add_enabled(redo, egui::Button::new("Redo (Ctrl+Y)")).clicked() {
            if let Some(d) = app.doc_mut() { d.redo(); }
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Clear Selection (Esc)").clicked() {
            if let Some(d) = app.doc_mut() { d.selection.clear(); }
            ui.close_menu();
        }
        if ui.button("Invert Selection").clicked() {
            if let Some(d) = app.doc_mut() { d.selection.invert(); }
            ui.close_menu();
        }
    });
}

fn image_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("Image", |ui| {
        if ui.button("Image Size…").clicked() {
            app.panels.show_image_size = true;
            ui.close_menu();
        }
        if ui.button("Canvas Size…").clicked() {
            app.panels.show_canvas_size = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Flip Horizontal").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() {
                    if let Some(p) = l.as_pixel_mut() {
                        let mut out = p.clone();
                        crate::filters::apply_in_place(&crate::filters::distort::FlipHorizontal, &mut out);
                        *p = out;
                    }
                }
            }
            ui.close_menu();
        }
        if ui.button("Flip Vertical").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() {
                    if let Some(p) = l.as_pixel_mut() {
                        let mut out = p.clone();
                        crate::filters::apply_in_place(&crate::filters::distort::FlipVertical, &mut out);
                        *p = out;
                    }
                }
            }
            ui.close_menu();
        }
        if ui.button("Rotate 90°").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() {
                    if let Some(p) = l.as_pixel_mut() {
                        let mut out = crate::render::pixel_buffer::PixelBuffer::new(p.height(), p.width());
                        crate::filters::apply_in_place(&crate::filters::distort::Rotate90, &mut out);
                        *p = out;
                    }
                }
            }
            ui.close_menu();
        }
    });
}

fn layer_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("Layer", |ui| {
        if ui.button("New Layer").clicked() {
            if let Some(d) = app.doc_mut() { d.add_pixel_layer(format!("Layer {}", d.layer_count())); }
            ui.close_menu();
        }
        if ui.button("Duplicate Layer").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(id) = d.active_layer { d.duplicate_layer(id); }
            }
            ui.close_menu();
        }
        if ui.button("Delete Layer").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(id) = d.active_layer { d.delete_layer(id); }
            }
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Flatten Image").clicked() {
            if let Some(d) = app.doc_mut() { d.flatten(); }
            ui.close_menu();
        }
        if ui.button("Add Layer Mask").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() { l.add_mask(); }
            }
            ui.close_menu();
        }
    });
}

fn filter_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("Filter", |ui| {
        if ui.button("Gaussian Blur…").clicked() {
            app.panels.show_blur = true;
            ui.close_menu();
        }
        if ui.button("Sharpen…").clicked() {
            app.panels.show_sharpen = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Brightness/Contrast…").clicked() {
            app.panels.show_brightness_contrast = true;
            ui.close_menu();
        }
        if ui.button("Hue/Saturation…").clicked() {
            app.panels.show_hue_saturation = true;
            ui.close_menu();
        }
        if ui.button("Levels…").clicked() {
            app.panels.show_levels = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Invert").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() {
                    if let Some(p) = l.as_pixel_mut() {
                        let mut out = p.clone();
                        crate::filters::apply_in_place(&crate::filters::adjustments::Invert, &mut out);
                        *p = out;
                    }
                }
            }
            ui.close_menu();
        }
        if ui.button("Grayscale").clicked() {
            if let Some(d) = app.doc_mut() {
                if let Some(l) = d.active_layer_mut() {
                    if let Some(p) = l.as_pixel_mut() {
                        let mut out = p.clone();
                        crate::filters::apply_in_place(&crate::filters::adjustments::Grayscale, &mut out);
                        *p = out;
                    }
                }
            }
            ui.close_menu();
        }
        if ui.button("Posterize…").clicked() {
            app.panels.show_posterize = true;
            ui.close_menu();
        }
        if ui.button("Threshold…").clicked() {
            app.panels.show_threshold = true;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Add Noise…").clicked() {
            app.panels.show_noise = true;
            ui.close_menu();
        }
        if ui.button("Swirl…").clicked() {
            app.panels.show_swirl = true;
            ui.close_menu();
        }
    });
}

fn view_menu(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.menu_button("View", |ui| {
        if ui.button("Zoom In (Ctrl++)").clicked() {
            if let Some(d) = app.doc_mut() { d.canvas.zoom = (d.canvas.zoom * 1.25).min(32.0); d.canvas.fit_to_view = false; }
            ui.close_menu();
        }
        if ui.button("Zoom Out (Ctrl+-)").clicked() {
            if let Some(d) = app.doc_mut() { d.canvas.zoom = (d.canvas.zoom / 1.25).max(0.05); d.canvas.fit_to_view = false; }
            ui.close_menu();
        }
        if ui.button("Fit to View (Ctrl+1)").clicked() {
            if let Some(d) = app.doc_mut() { d.canvas.fit_to_view = true; }
            ui.close_menu();
        }
        if ui.button("Actual Pixels (Ctrl+0)").clicked() {
            if let Some(d) = app.doc_mut() { d.canvas.zoom = 1.0; d.canvas.fit_to_view = false; }
            ui.close_menu();
        }
        ui.separator();
        ui.checkbox(&mut app.panels.show_right, "Right Panel (Tab)");
    });
}

// Suppress unused warning
#[allow(dead_code)]
fn _button_unused() -> Button<'static> { Button::new("") }