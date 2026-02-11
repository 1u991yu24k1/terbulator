use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Parser for Kitty graphics protocol
/// Format: ESC _G<control data>;<payload>ESC \
/// Example: ESC _Gf=24,s=100,v=100;<base64 data>ESC \
pub struct KittyImageParser {
    buffer: Vec<u8>,
    in_sequence: bool,
}

impl KittyImageParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            in_sequence: false,
        }
    }

    /// Process a byte, returns Some(image) if complete image sequence was parsed
    pub fn process_byte(&mut self, byte: u8) -> Option<DynamicImage> {
        // Kitty graphics protocol: ESC _G ... ESC \
        // We detect ESC _G to start, ESC \ to end

        if !self.in_sequence {
            // Look for start sequence: ESC _G
            if byte == b'G' && self.buffer.last() == Some(&b'_') && self.buffer.len() >= 2 {
                if self.buffer[self.buffer.len() - 2] == 0x1b {
                    // Found start of Kitty graphics
                    self.in_sequence = true;
                    self.buffer.clear();
                    return None;
                }
            }
            self.buffer.push(byte);
            // Keep buffer small when not in sequence
            if self.buffer.len() > 100 {
                self.buffer.drain(0..50);
            }
            return None;
        }

        // In sequence - collect until ESC \
        self.buffer.push(byte);

        // Check for end sequence: ESC \ (0x1b 0x5c)
        let len = self.buffer.len();
        if len >= 2 && self.buffer[len - 2] == 0x1b && self.buffer[len - 1] == 0x5c {
            // Remove ESC \ from buffer
            self.buffer.truncate(len - 2);

            // Parse the complete sequence
            let result = self.parse_sequence();

            // Reset state
            self.in_sequence = false;
            self.buffer.clear();

            return result;
        }

        // Limit buffer size to prevent memory issues
        if self.buffer.len() > 10 * 1024 * 1024 {
            log::warn!("Kitty image sequence too large, aborting");
            self.in_sequence = false;
            self.buffer.clear();
        }

        None
    }

    fn parse_sequence(&self) -> Option<DynamicImage> {
        // Format: <control data>;<payload>
        // Control data: key=value,key=value,...
        // We mainly care about the payload (base64 encoded image)

        let seq_str = String::from_utf8_lossy(&self.buffer);

        // Split by semicolon
        let parts: Vec<&str> = seq_str.splitn(2, ';').collect();
        if parts.len() < 2 {
            log::warn!("Invalid Kitty image format: no payload separator");
            return None;
        }

        let _control_data = parts[0];
        let payload = parts[1];

        // Decode base64
        let image_data = match STANDARD.decode(payload.trim()) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to decode Kitty image base64: {}", e);
                return None;
            }
        };

        // Try to load as PNG first (most common), then other formats
        match image::load_from_memory(&image_data) {
            Ok(img) => {
                log::info!(
                    "Successfully loaded Kitty image: {}x{}",
                    img.width(),
                    img.height()
                );
                Some(img)
            }
            Err(e) => {
                log::warn!("Failed to load Kitty image: {}", e);
                None
            }
        }
    }

    /// Reset parser state
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.in_sequence = false;
    }
}

impl Default for KittyImageParser {
    fn default() -> Self {
        Self::new()
    }
}
