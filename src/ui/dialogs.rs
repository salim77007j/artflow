//! Modal dialogs: Image size, canvas size, filter dialogs, settings.

use crate::app::ArtFlowApp;
use eframe::egui;

pub fn show(ctx: &egui::Context, app: &mut ArtFlowApp) {
    if app.panels.show_image_size { image_size_dialog(ctx, app); }
    if app.panels.show_canvas_size { canvas_size_dialog(ctx, app); }
    if app.panels.show_blur { blur_dialog(ctx, app); }
    if app.panels.show_sharpen { sharpen_dialog(ctx, app); }
    if app.panels.show_brightness_contrast { brightness_contrast_dialog(ctx, app); }
    if app.panels.show_hue_saturation { hue_sat_dialog(ctx, app); }
    if app.panels.show_levels { levels_dialog(ctx, app); }
    if app.panels.show_posterize { posterize_dialog(ctx, app); }
    if app.panels.show_threshold { threshold_dialog(ctx, app); }
    if app.panels.show_noise { noise_dialog(ctx, app); }
    if app.panels.show_swirl { swirl_dialog(ctx, app); }
    if app.panels.show_settings { settings_dialog(ctx, app); }
}

fn image_size_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Image Size").resizable(false).collapsible(false).show(ctx, |ui| {
        let Some(doc) = app.doc() else { return; };
        let mut w = doc.width() as i32;
        let mut h = doc.height() as i32;
        ui.horizontal(|ui| { ui.label("Width"); ui.add(egui::DragValue::new(&mut w).range(1..=16384)); });
        ui.horizontal(|ui| { ui.label("Height"); ui.add(egui::DragValue::new(&mut h).range(1..=16384)); });
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.layers.iter_mut().find(|l| matches!(l.kind, crate::document::layer::LayerKind::Pixel(_))) {
                        if let Some(p) = l.as_pixel_mut() { p.resize_scaled(w as u32, h as u32); }
                    }
                    d.resize_canvas(w as u32, h as u32);
                }
                app.panels.show_image_size = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_image_size = false; }
        });
    });
}

fn canvas_size_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut w = app.panels.modal_new_w.max(1);
    let mut h = app.panels.modal_new_h.max(1);
    egui::Window::new("Canvas Size").resizable(false).collapsible(false).show(ctx, |ui| {
        ui.horizontal(|ui| { ui.label("Width"); ui.add(egui::DragValue::new(&mut w).range(1..=16384)); });
        ui.horizontal(|ui| { ui.label("Height"); ui.add(egui::DragValue::new(&mut h).range(1..=16384)); });
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() { d.resize_canvas(w, h); }
                app.panels.modal_new_w = w;
                app.panels.modal_new_h = h;
                app.panels.show_canvas_size = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_canvas_size = false; }
        });
    });
}

fn blur_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut radius = app.panels.blur_radius;
    egui::Window::new("Gaussian Blur").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut radius, 0.0_f32..=64.0_f32).text("Radius"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::blur::GaussianBlur { radius: radius as u32 });
                app.panels.blur_radius = radius;
                app.panels.show_blur = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_blur = false; }
        });
    });
}

fn sharpen_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut amount = app.panels.sharpen_amount;
    egui::Window::new("Sharpen").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut amount, 0.0_f32..=4.0_f32).text("Amount"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::blur::Sharpen { amount });
                app.panels.sharpen_amount = amount;
                app.panels.show_sharpen = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_sharpen = false; }
        });
    });
}

fn brightness_contrast_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut brightness = app.panels.bc_brightness;
    let mut contrast = app.panels.bc_contrast;
    egui::Window::new("Brightness / Contrast").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut brightness, -1.0_f32..=1.0_f32).text("Brightness"));
        ui.add(egui::Slider::new(&mut contrast, -1.0_f32..=1.0_f32).text("Contrast"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::adjustments::BrightnessContrast { brightness, contrast });
                app.panels.bc_brightness = brightness;
                app.panels.bc_contrast = contrast;
                app.panels.show_brightness_contrast = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_brightness_contrast = false; }
        });
    });
}

fn hue_sat_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut hue = app.panels.hue_shift;
    let mut sat = app.panels.sat_mult;
    let mut val = app.panels.val_mult;
    egui::Window::new("Hue / Saturation").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut hue, -1.0_f32..=1.0_f32).text("Hue"));
        ui.add(egui::Slider::new(&mut sat, 0.0_f32..=2.0_f32).text("Saturation"));
        ui.add(egui::Slider::new(&mut val, 0.0_f32..=2.0_f32).text("Lightness"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::adjustments::HueSaturation { hue_shift: hue, sat_mult: sat, val_mult: val });
                app.panels.hue_shift = hue;
                app.panels.sat_mult = sat;
                app.panels.val_mult = val;
                app.panels.show_hue_saturation = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_hue_saturation = false; }
        });
    });
}

fn levels_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut black = app.panels.levels_black;
    let mut gamma = app.panels.levels_gamma;
    let mut white = app.panels.levels_white;
    egui::Window::new("Levels").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut black, 0.0_f32..=1.0_f32).text("Black"));
        ui.add(egui::Slider::new(&mut gamma, 0.1..=4.0_f32).text("Gamma"));
        ui.add(egui::Slider::new(&mut white, 0.0_f32..=1.0_f32).text("White"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::adjustments::Levels { black, gamma, white });
                app.panels.levels_black = black;
                app.panels.levels_gamma = gamma;
                app.panels.levels_white = white;
                app.panels.show_levels = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_levels = false; }
        });
    });
}

fn posterize_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut levels = app.panels.posterize_levels;
    egui::Window::new("Posterize").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut levels, 2..=32).text("Levels"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::adjustments::Posterize { levels });
                app.panels.posterize_levels = levels;
                app.panels.show_posterize = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_posterize = false; }
        });
    });
}

fn threshold_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut threshold = app.panels.threshold_value;
    egui::Window::new("Threshold").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut threshold, 0.0_f32..=1.0_f32).text("Threshold"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::adjustments::Threshold { threshold });
                app.panels.threshold_value = threshold;
                app.panels.show_threshold = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_threshold = false; }
        });
    });
}

fn noise_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut amount = app.panels.noise_amount;
    egui::Window::new("Add Noise").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut amount, 0.0_f32..=1.0_f32).text("Amount"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::noise::AddNoise { amount });
                app.panels.noise_amount = amount;
                app.panels.show_noise = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_noise = false; }
        });
    });
}

fn swirl_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    let mut strength = app.panels.swirl_strength;
    egui::Window::new("Swirl").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut strength, -10.0_f32..=10.0_f32).text("Strength"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                apply_filter(app, &crate::filters::distort::Swirl { strength });
                app.panels.swirl_strength = strength;
                app.panels.show_swirl = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_swirl = false; }
        });
    });
}

fn settings_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Settings").resizable(false).show(ctx, |ui| {
        ui.label("Application settings");
        ui.separator();
        let mut show = app.panels.show_right;
        if ui.checkbox(&mut show, "Show right panels").changed() {
            app.panels.show_right = show;
        }
        ui.add_space(4.0);
        ui.label("Built with Rust + egui");
        ui.label("© ArtFlow Studio");
        if ui.button("Close").clicked() { app.panels.show_settings = false; }
    });
}

/// Apply a filter to the active layer (helper to centralise the borrow logic).
fn apply_filter(app: &mut ArtFlowApp, filter: &dyn crate::filters::Filter) {
    if let Some(d) = app.doc_mut() {
        if let Some(l) = d.active_layer_mut() {
            if let Some(p) = l.as_pixel_mut() {
                let mut out = p.clone();
                crate::filters::apply_in_place(filter, &mut out);
                *p = out;
            }
        }
    }
}