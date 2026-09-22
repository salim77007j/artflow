//! ArtFlow project format (.aflow) — a JSON wrapper that round-trips the
//! entire document tree.

use crate::app::ArtFlowApp;
use crate::document::Document;
use std::path::Path;

pub const MAGIC: &str = "ARTFLOW1";

pub fn save(app: &mut ArtFlowApp, path: &Path) -> anyhow::Result<()> {
    let Some(doc) = app.doc() else { return Ok(()); };
    let json = serde_json::to_vec(doc)?;
    let mut f = std::fs::File::create(path)?;
    use std::io::Write;
    f.write_all(MAGIC.as_bytes())?;
    f.write_all(&(json.len() as u64).to_le_bytes())?;
    f.write_all(&json)?;
    Ok(())
}

pub fn load(app: &mut ArtFlowApp, path: &Path) -> anyhow::Result<()> {
    let data = std::fs::read(path)?;
    if data.len() < MAGIC.len() + 8 { return Err(anyhow::anyhow!("Truncated project file")); }
    if &data[..MAGIC.len()] != MAGIC.as_bytes() {
        return Err(anyhow::anyhow!("Not an ArtFlow project"));
    }
    let len = u64::from_le_bytes(data[MAGIC.len()..MAGIC.len() + 8].try_into().unwrap()) as usize;
    let body = &data[MAGIC.len() + 8..MAGIC.len() + 8 + len];
    let doc: Document = serde_json::from_slice(body)?;
    let id = app.store.push(doc);
    app.active_doc = Some(id);
    Ok(())
}