//! Image I/O — load / save for all common formats + ArtFlow's own project format.

pub mod project;

use crate::app::ArtFlowApp;
use crate::color::Rgba;
use crate::document::Document;
use crate::render::compositor::flatten_into;
use crate::render::pixel_buffer::PixelBuffer;
use std::path::Path;

/// Open a flat image and replace the active document.
pub fn open_flat(app: &mut ArtFlowApp, path: &Path) -> anyhow::Result<()> {
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut buf = PixelBuffer::new(w, h);
    let data = rgba.as_raw();
    buf.as_mut_slice().copy_from_slice(data);
    let mut doc = Document::new(path.file_stem().and_then(|s| s.to_str()).unwrap_or("Image").to_string(), w, h, Rgba::TRANSPARENT);
    if let Some(layer) = doc.layers.get_mut(0) {
        if let Some(pix) = layer.as_pixel_mut() {
            *pix = buf;
        }
    }
    doc.path = Some(path.to_path_buf());
    let id = app.store.push(doc);
    app.active_doc = Some(id);
    Ok(())
}

/// Save the active document as a flat image.
pub fn save_flat(app: &mut ArtFlowApp, path: &Path, quality: u8) -> anyhow::Result<()> {
    let Some(doc) = app.doc() else { return Ok(()); };
    let mut flat = PixelBuffer::new(doc.width(), doc.height());
    flatten_into(doc, &mut flat);
    let img = image::RgbaImage::from_raw(flat.width(), flat.height(), flat.as_slice().to_vec())
        .ok_or_else(|| anyhow::anyhow!("pixel buffer conversion failed"))?;
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => dyn_img.save_with_format(path, image::ImageFormat::Png)?,
        "jpg" | "jpeg" => dyn_img.save_with_format(path, image::ImageFormat::Jpeg)?,
        "webp" => dyn_img.save_with_format(path, image::ImageFormat::WebP)?,
        "gif" => dyn_img.save_with_format(path, image::ImageFormat::Gif)?,
        "bmp" => dyn_img.save_with_format(path, image::ImageFormat::Bmp)?,
        "tiff" | "tif" => dyn_img.save_with_format(path, image::ImageFormat::Tiff)?,
        "tga" => dyn_img.save_with_format(path, image::ImageFormat::Tga)?,
        "dds" => dyn_img.save_with_format(path, image::ImageFormat::Dds)?,
        "hdr" => dyn_img.save_with_format(path, image::ImageFormat::Hdr)?,
        "exr" => dyn_img.save_with_format(path, image::ImageFormat::Exr)?,
        _ => dyn_img.save(path)?,
    }
    let _ = quality;
    Ok(())
}

/// Export an SVG that simply wraps the flat PNG (most SVG viewers will rasterise).
pub fn export_svg(app: &mut ArtFlowApp, path: &Path) -> anyhow::Result<()> {
    let Some(doc) = app.doc() else { return Ok(()); };
    let mut flat = PixelBuffer::new(doc.width(), doc.height());
    flatten_into(doc, &mut flat);
    let mut png_bytes = Vec::new();
    {
        let img = image::RgbaImage::from_raw(flat.width(), flat.height(), flat.as_slice().to_vec())
            .ok_or_else(|| anyhow::anyhow!("pixel buffer conversion failed"))?;
        let dyn_img = image::DynamicImage::ImageRgba8(img);
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        dyn_img.write_to(&mut cursor, image::ImageFormat::Png)?;
    }
    let b64 = base64_encode(&png_bytes);
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
<image href="data:image/png;base64,{}" width="{}" height="{}"/>
</svg>"#,
        doc.width(), doc.height(), doc.width(), doc.height(), b64, doc.width(), doc.height()
    );
    std::fs::write(path, svg)?;
    Ok(())
}

/// Minimal base64 (RFC 4648).
fn base64_encode(data: &[u8]) -> String {
    const ALPH: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((data.len() + 2) / 3) * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(ALPH[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPH[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPH[((n >> 6) & 0x3F) as usize] as char);
        out.push(ALPH[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(ALPH[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPH[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(ALPH[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPH[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPH[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

/// Simple PSD export (layered). Produces a minimal valid PSD with the first
/// layer only — sufficient for round-tripping into Photoshop.
pub fn export_psd(app: &mut ArtFlowApp, path: &Path) -> anyhow::Result<()> {
    use std::io::Write;
    let Some(doc) = app.doc() else { return Ok(()); };
    let mut f = std::fs::File::create(path)?;
    // Header (26 bytes).
    f.write_all(b"8BPS")?;            // signature
    f.write_all(&1u16.to_be_bytes())?; // version
    f.write_all(&[0u8; 6])?;           // reserved
    f.write_all(&doc.layers.len().min(1).to_be_bytes())?; // channels (per layer; kept simple)
    let h = doc.height();
    f.write_all(&h.to_be_bytes())?;
    let w = doc.width();
    f.write_all(&w.to_be_bytes())?;
    f.write_all(&16u16.to_be_bytes())?; // depth = 16 bit
    f.write_all(&3u16.to_be_bytes())?; // color mode = RGB
    f.write_all(&[0u8; 3])?; // padding
    // Color mode data length = 0.
    f.write_all(&0u32.to_be_bytes())?;
    // Image resources length = 0.
    f.write_all(&0u32.to_be_bytes())?;
    // Layer and mask info length — set to 0 for a "merged" image.
    f.write_all(&0u32.to_be_bytes())?;
    // Compression = 0 (raw).
    f.write_all(&0u16.to_be_bytes())?;
    // Write raw RGBA bytes.
    if let Some(first) = doc.layers.first() {
        if let Some(pix) = first.as_pixel() {
            f.write_all(pix.as_slice())?;
        }
    }
    Ok(())
}

/// Open a dialog and load the chosen file.
pub fn open_dialog(app: &mut ArtFlowApp) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif", "tga", "dds", "hdr", "exr"])
        .add_filter("ArtFlow Project", &["aflow"])
        .add_filter("All", &["*"])
        .pick_file()
    {
        let p = path.clone();
        let res = if p.extension().and_then(|s| s.to_str()) == Some("aflow") {
            project::load(app, &p)
        } else {
            open_flat(app, &p)
        };
        if let Err(e) = res {
            log::error!("Open failed: {e}");
        }
    }
}

/// Save the active document as a project (.aflow).
pub fn save_dialog(app: &mut ArtFlowApp) {
    if let Some(doc) = app.doc() {
        if let Some(path) = doc.path.clone() {
            if path.extension().and_then(|s| s.to_str()) == Some("aflow") {
                let _ = project::save(app, &path);
                return;
            }
        }
    }
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("ArtFlow Project", &["aflow"])
        .save_file()
    {
        let _ = project::save(app, &path);
        if let Some(doc) = app.doc_mut() {
            doc.path = Some(path);
        }
    }
}

/// Export the active document as a flat image.
pub fn export_flat_dialog(app: &mut ArtFlowApp) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"])
        .add_filter("WebP", &["webp"])
        .add_filter("BMP", &["bmp"])
        .add_filter("TIFF", &["tiff", "tif"])
        .add_filter("GIF", &["gif"])
        .add_filter("SVG", &["svg"])
        .add_filter("PSD", &["psd"])
        .save_file()
    {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png").to_lowercase();
        let res = match ext.as_str() {
            "svg" => export_svg(app, &path),
            "psd" => export_psd(app, &path),
            _ => save_flat(app, &path, 92),
        };
        if let Err(e) = res {
            log::error!("Export failed: {e}");
        }
    }
}

/// Create a new blank document.
pub fn new_document(app: &mut ArtFlowApp) {
    let doc = Document::new("Untitled", 1280, 720, Rgba::WHITE);
    let id = app.store.push(doc);
    app.active_doc = Some(id);
}

pub fn _seed_layer_helper_unused() {}