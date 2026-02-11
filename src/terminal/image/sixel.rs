use image::{DynamicImage, RgbaImage};

/// Parser for Sixel graphics protocol
/// Format: ESC P q ... ESC \
/// Sixel is a raster graphics format
pub struct SixelImageParser {
    buffer: Vec<u8>,
    in_sequence: bool,
}

impl SixelImageParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            in_sequence: false,
        }
    }

    /// Process a byte, returns Some(image) if complete image sequence was parsed
    pub fn process_byte(&mut self, byte: u8) -> Option<DynamicImage> {
        // Sixel graphics protocol: ESC P q ... ESC \
        // ESC P is Device Control String (DCS)
        // 'q' indicates sixel mode

        if !self.in_sequence {
            // Look for start sequence: ESC P q
            if byte == b'q' {
                let len = self.buffer.len();
                if len >= 2 && self.buffer[len - 1] == b'P' && self.buffer[len - 2] == 0x1b {
                    // Found start of sixel
                    self.in_sequence = true;
                    self.buffer.clear();
                    return None;
                }
            }
            self.buffer.push(byte);
            // Keep buffer small
            if self.buffer.len() > 100 {
                self.buffer.drain(0..50);
            }
            return None;
        }

        // In sequence - collect until ESC \
        self.buffer.push(byte);

        // Check for end sequence: ESC \ (0x1b 0x5c) or ST (String Terminator)
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

        // Limit buffer size
        if self.buffer.len() > 10 * 1024 * 1024 {
            log::warn!("Sixel sequence too large, aborting");
            self.in_sequence = false;
            self.buffer.clear();
        }

        None
    }

    fn parse_sequence(&self) -> Option<DynamicImage> {
        // Simplified sixel parser
        // Full implementation would be quite complex
        // For now, we'll create a placeholder

        let seq_str = String::from_utf8_lossy(&self.buffer);

        log::debug!("Parsing sixel sequence, length: {}", seq_str.len());

        // Parse sixel data (simplified)
        // Sixel format:
        // - "#<color>;<mode>;<r>;<g>;<b>" - define color
        // - "<data>" - sixel data (6 vertical pixels per byte)
        // - "$" - carriage return
        // - "-" - newline

        let mut width = 0;
        let mut height = 0;
        let mut x = 0;
        let mut y = 0;

        // Color palette (256 colors max for simplicity)
        let mut palette: Vec<[u8; 3]> = vec![[0, 0, 0]; 256];
        // Initialize with default VT340 palette
        for i in 0..16 {
            palette[i] = Self::default_color(i);
        }

        let mut current_color = 0;

        // Parse character by character
        let chars: Vec<char> = seq_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            match c {
                '#' => {
                    // Color definition: #<Pc>;<Pu>;<Px>;<Py>;<Pz>
                    i += 1;
                    let mut params = Vec::new();
                    let mut num_str = String::new();

                    while i < chars.len() {
                        let ch = chars[i];
                        if ch.is_ascii_digit() {
                            num_str.push(ch);
                        } else if ch == ';' {
                            if let Ok(n) = num_str.parse::<u8>() {
                                params.push(n);
                            }
                            num_str.clear();
                        } else {
                            if !num_str.is_empty() {
                                if let Ok(n) = num_str.parse::<u8>() {
                                    params.push(n);
                                }
                            }
                            i -= 1; // Back up to process this char in main loop
                            break;
                        }
                        i += 1;
                    }

                    // Apply color definition
                    if params.len() >= 5 {
                        let color_idx = params[0] as usize;
                        // params[1] is color coordination system (2=RGB)
                        let r = (params[2] as f32 / 100.0 * 255.0) as u8;
                        let g = (params[3] as f32 / 100.0 * 255.0) as u8;
                        let b = (params[4] as f32 / 100.0 * 255.0) as u8;
                        if color_idx < palette.len() {
                            palette[color_idx] = [r, g, b];
                        }
                        current_color = color_idx;
                    } else if !params.is_empty() {
                        // Just color selection
                        current_color = params[0] as usize;
                    }
                }
                '$' => {
                    // Carriage return
                    x = 0;
                }
                '-' => {
                    // Newline
                    x = 0;
                    y += 6; // Sixel row is 6 pixels high
                }
                '?' | '@'..='~' => {
                    // Sixel data byte
                    // Each byte represents 6 vertical pixels
                    x += 1;
                    if x > width {
                        width = x;
                    }
                    if y + 6 > height {
                        height = y + 6;
                    }
                }
                _ => {
                    // Ignore other characters
                }
            }

            i += 1;
        }

        // For now, create a placeholder image
        // Full sixel rendering would require pixel-by-pixel rendering
        if width > 0 && height > 0 {
            log::info!("Sixel image parsed: estimated {}x{}", width, height);

            // Create a simple placeholder image
            let img = RgbaImage::from_fn(width.min(800) as u32, height.min(600) as u32, |x, y| {
                // Simple gradient pattern as placeholder
                let r = ((x % 256) as u8).wrapping_add((y % 256) as u8);
                let g = ((y % 256) as u8);
                let b = ((x % 128) as u8).wrapping_mul(2);
                image::Rgba([r, g, b, 255])
            });

            Some(DynamicImage::ImageRgba8(img))
        } else {
            log::warn!("Failed to parse sixel: invalid dimensions");
            None
        }
    }

    fn default_color(index: usize) -> [u8; 3] {
        // Default VT340 color palette
        match index {
            0 => [0, 0, 0],       // Black
            1 => [51, 102, 179],  // Blue
            2 => [204, 51, 51],   // Red
            3 => [51, 204, 51],   // Green
            4 => [204, 51, 204],  // Magenta
            5 => [51, 204, 204],  // Cyan
            6 => [204, 204, 51],  // Yellow
            7 => [229, 229, 229], // White (Gray 90%)
            8 => [127, 127, 127], // Gray 50%
            9 => [179, 179, 255], // Light Blue
            10 => [255, 179, 179], // Light Red
            11 => [179, 255, 179], // Light Green
            12 => [255, 179, 255], // Light Magenta
            13 => [179, 255, 255], // Light Cyan
            14 => [255, 255, 179], // Light Yellow
            15 => [255, 255, 255], // Bright White
            _ => [128, 128, 128],  // Default gray
        }
    }

    /// Reset parser state
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.in_sequence = false;
    }
}

impl Default for SixelImageParser {
    fn default() -> Self {
        Self::new()
    }
}
