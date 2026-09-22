# ArtFlow — a modern, professional image editor in Rust

ArtFlow is a lightweight, fast, GPU-accelerated image editor and digital drawing studio written entirely in Rust. It's inspired by Photoshop and Krita, but designed to feel modern, stable and crash-resistant — even on large canvases with many layers and heavy edits.

## Features

- **Layers** — add, delete, duplicate, reorder; per-layer opacity, visibility, lock, blend modes (Normal, Multiply, Screen, Overlay, Darken, Lighten, Color Dodge, Color Burn, Soft Light, Hard Light, Difference, Exclusion, Hue, Saturation, Color, Luminosity, Add, Subtract); layer masks
- **Brush engine** — round / square brush, eraser, full per-stroke settings: size, hardness, opacity, flow, spacing; smooth interpolation
- **Selection tools** — rectangle, ellipse, lasso (placeholder), magic wand with adjustable tolerance, invert, feather
- **Transforms** — move, crop (interactive), flip horizontal / vertical, rotate 90°, free transform (placeholder)
- **Color tools** — color wheel with hue/saturation picker, swatches, hex input, brush stamp preview
- **Filters** — Gaussian blur, box blur, sharpen, brightness/contrast, hue/saturation, levels, invert, grayscale, posterize, threshold, add noise, Gaussian noise, swirl, wave, rotate 90°, flip H/V
- **Shape and text tools** — rectangle, ellipse, line; text tool reserved for future expansion
- **Canvas navigation** — zoom, pan, fit-to-view, actual-pixels; smooth scrolling and mouse-wheel zoom
- **Multi-document** — `DocumentStore` keeps every open document in memory
- **Full undo / redo** — history stack with N=256 operations and per-operation snapshot data
- **Import / export** — PNG, JPEG, WebP, GIF, BMP, TIFF, TGA, DDS, HDR, OpenEXR, plus ArtFlow's own `.aflow` project format, an SVG exporter (PNG embedded as base64) and a minimal PSD writer
- **GPU-accelerated rendering** — `egui` is built on `wgpu`/`glow` and the canvas is rendered as an `egui::TextureHandle` so the GPU composites every frame
- **Keyboard shortcuts** — B/E/V/M/L/W/P/G/I/T/U/O/C, Ctrl+Z/Shift+Z/Ctrl+S/Ctrl+O/Ctrl+N/Ctrl+E, Ctrl +/-/0/1, Tab, Esc
- **Modern UI** — custom light theme, smooth corners, transparent panels, animated marching-ants selection

## Architecture

```
src/
├── main.rs              # Entry point
├── app.rs               # Top-level state + eframe::App implementation
├── color.rs             # Color types, palette, picker state
├── document/            # Document, Layer, History, Selection, Transform, Blend
├── tools/               # Brush, eraser, fill, selection, shape, transform, eyedropper, …
├── render/              # Pixel buffer, compositor, canvas widget
├── filters/             # Blur, sharpen, adjustments, distort, noise
├── io/                  # Project save/load, flat-image import/export, SVG, PSD
└── ui/                  # Top menu, left toolbar, right panels, dialogs
```

## Building

```bash
# Debug build
cargo build

# Release build (optimised, stripped)
cargo build --release

# Run
cargo run --release
```

Tested targets: `x86_64-unknown-linux-gnu`. Should compile on Windows and macOS as well — `eframe` and `rfd` are cross-platform.

## Keyboard shortcuts

| Action                     | Shortcut            |
|----------------------------|---------------------|
| Brush                      | B                   |
| Eraser                     | E                   |
| Pencil                     | P                   |
| Move                       | V                   |
| Rectangle select           | M                   |
| Ellipse select             | L                   |
| Magic wand                 | W                   |
| Crop                       | C                   |
| Eyedropper                 | I                   |
| Gradient                   | G                   |
| Text                       | T                   |
| Rectangle (shape)         | U                   |
| Ellipse (shape)            | O                   |
| Fill / bucket              | Shift+F             |
| Undo                       | Ctrl+Z              |
| Redo                       | Ctrl+Y / Ctrl+Shift+Z|
| Save                       | Ctrl+S              |
| Open                       | Ctrl+O              |
| New                        | Ctrl+N              |
| Export flat                | Ctrl+E              |
| Zoom in / out              | Ctrl++ / Ctrl+-     |
| Actual pixels              | Ctrl+0              |
| Fit to view                | Ctrl+1              |
| Toggle right panels        | Tab                 |
| Clear selection            | Esc                 |

## License

Dual-licensed under MIT OR Apache-2.0. See the source headers for details.