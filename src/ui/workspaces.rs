//! Workspace state (saved layouts). Just a stub for now — we're focused on
//! shipping a working core.

#[derive(Debug, Default)]
pub struct WorkspaceState {
    pub name: String,
}

impl WorkspaceState {
    pub fn save_current(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }
}