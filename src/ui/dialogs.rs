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
        ui.horizontal(|ui| { ui.label("Width"); ui.add(egui::DragValue::new(&mut w).clamp_range(1..=16384)); });
        ui.horizontal(|ui| { ui.label("Height"); ui.add(egui::DragValue::new(&mut h).clamp_range(1..=16384)); });
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
    egui::Window::new("Canvas Size").resizable(false).collapsible(false).show(ctx, |ui| {
        let mut w = app.panels.modal_new_w.max(1);
        let mut h = app.panels.modal_new_h.max(1);
        ui.horizontal(|ui| { ui.label("Width"); ui.add(egui::DragValue::new(&mut w).clamp_range(1..=16384)); });
        ui.horizontal(|ui| { ui.label("Height"); ui.add(egui::DragValue::new(&mut h).clamp_range(1..=16384)); });
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() { d.resize_canvas(w, h); }
                app.panels.show_canvas_size = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_canvas_size = false; }
        });
    });
}

fn blur_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Gaussian Blur").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.blur_radius, 0.0..=64.0).text("Radius"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(&crate::filters::blur::GaussianBlur { radius: app.panels.blur_radius as u32 }, &mut out);
                            *p = out;
                        }
                    }
                }
                app.panels.show_blur = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_blur = false; }
        });
    });
}

fn sharpen_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Sharpen").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.sharpen_amount, 0.0..=4.0).text("Amount"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(&crate::filters::blur::Sharpen { amount: app.panels.sharpen_amount }, &mut out);
                            *p = out;
                        }
                    }
                }
                app.panels.show_sharpen = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_sharpen = false; }
        });
    });
}

fn brightness_contrast_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Brightness / Contrast").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.bc_brightness, -1.0..=1.0).text("Brightness"));
        ui.add(egui::Slider::new(&mut app.panels.bc_contrast, -1.0..=1.0).text("Contrast"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::adjustments::BrightnessContrast {
                                    brightness: app.panels.bc_brightness,
                                    contrast: app.panels.bc_contrast,
                                },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_brightness_contrast = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_brightness_contrast = false; }
        });
    });
}

fn hue_sat_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Hue / Saturation").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.hue_shift, -1.0..=1.0).text("Hue"));
        ui.add(egui::Slider::new(&mut app.panels.sat_mult, 0.0..=2.0).text("Saturation"));
        ui.add(egui::Slider::new(&mut app.panels.val_mult, 0.0..=2.0).text("Lightness"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::adjustments::HueSaturation {
                                    hue_shift: app.panels.hue_shift,
                                    sat_mult: app.panels.sat_mult,
                                    val_mult: app.panels.val_mult,
                                },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_hue_saturation = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_hue_saturation = false; }
        });
    });
}

fn levels_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Levels").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.levels_black, 0.0..=1.0).text("Black"));
        ui.add(egui::Slider::new(&mut app.panels.levels_gamma, 0.1..=4.0).text("Gamma"));
        ui.add(egui::Slider::new(&mut app.panels.levels_white, 0.0..=1.0).text("White"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::adjustments::Levels {
                                    black: app.panels.levels_black,
                                    gamma: app.panels.levels_gamma,
                                    white: app.panels.levels_white,
                                },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_levels = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_levels = false; }
        });
    });
}

fn posterize_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Posterize").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.posterize_levels, 2..=32).text("Levels"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::adjustments::Posterize { levels: app.panels.posterize_levels },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_posterize = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_posterize = false; }
        });
    });
}

fn threshold_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Threshold").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.threshold_value, 0.0..=1.0).text("Threshold"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::adjustments::Threshold { threshold: app.panels.threshold_value },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_threshold = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_threshold = false; }
        });
    });
}

fn noise_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Add Noise").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.noise_amount, 0.0..=1.0).text("Amount"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::noise::AddNoise { amount: app.panels.noise_amount },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
                app.panels.show_noise = false;
            }
            if ui.button("Cancel").clicked() { app.panels.show_noise = false; }
        });
    });
}

fn swirl_dialog(ctx: &egui::Context, app: &mut ArtFlowApp) {
    egui::Window::new("Swirl").resizable(false).show(ctx, |ui| {
        ui.add(egui::Slider::new(&mut app.panels.swirl_strength, -10.0..=10.0).text("Strength"));
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Some(d) = app.doc_mut() {
                    if let Some(l) = d.active_layer_mut() {
                        if let Some(p) = l.as_pixel_mut() {
                            let mut out = p.clone();
                            crate::filters::apply_in_place(
                                &crate::filters::distort::Swirl { strength: app.panels.swirl_strength },
                                &mut out,
                            );
                            *p = out;
                        }
                    }
                }
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
        ui.checkbox(&mut app.panels.show_right, "Show right panels");
        ui.add_space(4.0);
        ui.label("Built with Rust + egui");
        ui.label("© ArtFlow Studio");
        if ui.button("Close").clicked() { app.panels.show_settings = false; }
    });
}