//! Left toolbar (vertical tool icons) + top secondary strip.

use crate::app::ArtFlowApp;
use crate::tools::ToolId;
use eframe::egui::{self, Color32, RichText};

pub fn show_left(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    let tools: &[(ToolId, &str)] = &[
        (ToolId::Move, "✥"),
        (ToolId::MarqueeRect, "▭"),
        (ToolId::MarqueeEllipse, "◯"),
        (ToolId::Lasso, "⌒"),
        (ToolId::MagicWand, "✦"),
        (ToolId::Crop, "⤢"),
        (ToolId::Eyedropper, "◉"),
        (ToolId::Brush, "🖌"),
        (ToolId::Pencil, "✏"),
        (ToolId::Eraser, "⌫"),
        (ToolId::Fill, "⏍"),
        (ToolId::Gradient, "▤"),
        (ToolId::ShapeRect, "▢"),
        (ToolId::ShapeEllipse, "⬭"),
        (ToolId::ShapeLine, "╱"),
        (ToolId::Text, "T"),
    ];
    ui.add_space(6.0);
    for (id, icon) in tools {
        let active = app.active_tool == *id;
        let btn = ui.add(
            egui::Button::new(RichText::new(*icon).size(18.0))
                .min_size(egui::vec2(40.0, 40.0))
                .fill(if active { Color32::from_rgb(220, 230, 248) } else { Color32::TRANSPARENT })
                .stroke(egui::Stroke::new(
                    1.0,
                    if active { Color32::from_rgb(90, 140, 230) } else { Color32::from_rgb(220, 224, 232) },
                )),
        );
        if btn.clicked() {
            app.set_tool(*id);
        }
        if btn.hovered() {
            btn.on_hover_text(format!("{} ({})", id.label(), id.shortcut()));
        }
    }
    ui.add_space(6.0);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.available_height()),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space(8.0);
            // Foreground / Background color swatch (mirrors Photoshop's bottom pair).
            let fg = app.color.foreground.to_rgba8();
            let bg = app.color.background.to_rgba8();
            let fg_col = Color32::from_rgb(fg.0, fg.1, fg.2);
            let bg_col = Color32::from_rgb(bg.0, bg.1, bg.2);
            let (rect_fg, rect_bg) = color_pair(ui);
            ui.painter().rect_filled(rect_fg, 0.0, fg_col);
            ui.painter().rect_stroke(rect_fg, 0.0, egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)));
            ui.painter().rect_filled(rect_bg, 0.0, bg_col);
            ui.painter().rect_stroke(rect_bg, 0.0, egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)));
            if ui.input(|i| i.modifiers.shift) && ui.input(|i| i.pointer.primary_clicked()) {
                app.color.swap();
            } else if ui.input(|i| i.pointer.primary_clicked()) && rect_bg.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                app.color.foreground = app.color.background;
            }
        },
    );
}

fn color_pair(ui: &egui::Ui) -> (egui::Rect, egui::Rect) {
    let center = ui.cursor().min + egui::vec2(ui.available_width() * 0.5, 12.0);
    let rect_fg = egui::Rect::from_center_size(center + egui::vec2(-6.0, -6.0), egui::vec2(22.0, 22.0));
    let rect_bg = egui::Rect::from_center_size(center + egui::vec2(6.0, 6.0), egui::vec2(22.0, 22.0));
    (rect_fg, rect_bg)
}

pub fn show_secondary_strip(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    ui.label(egui::RichText::new(format!("{} — {}", app.active_tool.label(), app.active_tool.shortcut())).strong());
    ui.separator();
    ui.label("Size:");
    ui.add(egui::DragValue::new(&mut app.color.brush_size).clamp_range(0.5..=512.0).speed(1.0));
    ui.label("Hardness:");
    ui.add(egui::DragValue::new(&mut app.color.brush_hardness).clamp_range(0.0..=1.0).speed(0.01));
    ui.label("Opacity:");
    ui.add(egui::DragValue::new(&mut app.color.brush_opacity).clamp_range(0.0..=1.0).speed(0.01));
    ui.label("Flow:");
    ui.add(egui::DragValue::new(&mut app.color.brush_flow).clamp_range(0.0..=1.0).speed(0.01));
}

pub fn show_topbar_right(ui: &mut egui::Ui, app: &mut ArtFlowApp) {
    if let Some(doc) = app.doc() {
        ui.label(format!("Zoom: {:.0}%", doc.canvas.zoom * 100.0));
        if ui.button("Fit").clicked() {
            if let Some(d) = app.doc_mut() { d.canvas.fit_to_view = true; }
        }
    }
    let _ = app;
}