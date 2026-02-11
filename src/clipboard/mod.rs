mod selection;

pub use selection::Selection;

use crate::utils::Result;
use arboard::Clipboard;
use log;

/// Clipboard manager for Copy/Paste operations
pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| crate::utils::TerbulatorError::io(format!("Failed to initialize clipboard: {}", e)))?;

        log::info!("Clipboard manager initialized");

        Ok(Self { clipboard })
    }

    /// Copy text to clipboard
    pub fn copy(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .map_err(|e| crate::utils::TerbulatorError::io(format!("Failed to copy to clipboard: {}", e)))?;

        log::debug!("Copied {} bytes to clipboard", text.len());

        Ok(())
    }

    /// Paste text from clipboard
    pub fn paste(&mut self) -> Result<String> {
        let text = self.clipboard
            .get_text()
            .map_err(|e| crate::utils::TerbulatorError::io(format!("Failed to paste from clipboard: {}", e)))?;

        log::debug!("Pasted {} bytes from clipboard", text.len());

        Ok(text)
    }
}
