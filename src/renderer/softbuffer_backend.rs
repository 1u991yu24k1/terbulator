use crate::renderer::backend::{BackendType, Color, CursorInfo, RenderBackend};
use crate::terminal::Grid;
use crate::utils::{Result, TerbulatorError};
use cosmic_text::{Attrs, Buffer, Color as CosmicColor, FontSystem, Metrics, Shaping, SwashCache};
use softbuffer::{Context, Surface};
use std::collections::HashMap;
use std::num::NonZeroU32;
use winit::window::Window;

/// Cache key for shaped glyphs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphCacheKey {
    ch: char,
    bold: bool,
}

pub struct SoftbufferBackend {
    surface: Surface<&'static Window, &'static Window>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    glyph_buffer_cache: HashMap<GlyphCacheKey, Buffer>,
    font_size: f32,
    cell_width: f32,
    cell_height: f32,
    width: u32,
    height: u32,
    buffer: Vec<u32>,
}

impl SoftbufferBackend {
    fn color_to_u32(color: Color) -> u32 {
        // softbuffer uses 0RGB format (or XRGB), top 8 bits are ignored
        // But we set alpha to 0xFF for compatibility
        0xFF000000 | ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }

    fn render_text_to_buffer(&mut self, grid: &mut Grid, cursor: CursorInfo) {
        let grid_cols = grid.cols();
        let grid_rows = grid.rows();

        // Note: Buffer is cleared in clear() method before rendering all panes
        // Don't clear here as it would erase other panes in multi-pane mode

        // Always do full redraw for simplicity and correctness
        // Differential rendering is complex with multi-pane rendering
        for row in 0..grid_rows {
            for col in 0..grid_cols {
                if let Some(cell) = grid.get(col, row) {
                    self.render_cell(col, row, cell);
                }
            }
        }

        // Draw cursor as an underline
        if cursor.visible && cursor.row < grid_rows && cursor.col < grid_cols {
            let x = (cursor.col as f32 * self.cell_width) as i32;
            let y = (cursor.row as f32 * self.cell_height) as i32;
            let cursor_height = 2; // Underline style cursor
            // Position cursor at about 80% down the cell height
            let cursor_y = y + (self.cell_height * 0.8) as i32;
            self.draw_rect(x, cursor_y, self.cell_width as i32, cursor_height, Color::WHITE);
        }

        // Clear dirty tracking after rendering
        grid.clear_dirty();
    }

    fn render_cell(&mut self, col: usize, row: usize, cell: &crate::terminal::grid::Cell) {
        let x = (col as f32 * self.cell_width) as i32;
        let y = (row as f32 * self.cell_height) as i32;

        // Determine colors (handle inverse)
        let (fg, bg) = if cell.attrs.inverse {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };

        // Draw background
        self.draw_rect(x, y, self.cell_width as i32, self.cell_height as i32, bg);

        // Draw character using cosmic-text
        if cell.ch != ' ' && cell.ch != '\0' {
            self.draw_char(x, y, cell.ch, fg, cell.attrs.bold);
        }
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        let width = self.width as i32;
        let height = self.height as i32;
        let color_u32 = Self::color_to_u32(color);

        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && px < width && py >= 0 && py < height {
                    let idx = (py * width + px) as usize;
                    if idx < self.buffer.len() {
                        self.buffer[idx] = color_u32;
                    }
                }
            }
        }
    }

    fn draw_rect_blend(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        let width = self.width as i32;
        let height = self.height as i32;
        let alpha = color.a as f32 / 255.0;

        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && px < width && py >= 0 && py < height {
                    let idx = (py * width + px) as usize;
                    if idx < self.buffer.len() {
                        let bg = self.buffer[idx];
                        let bg_r = ((bg >> 16) & 0xFF) as f32;
                        let bg_g = ((bg >> 8) & 0xFF) as f32;
                        let bg_b = (bg & 0xFF) as f32;

                        let r = (color.r as f32 * alpha + bg_r * (1.0 - alpha)) as u32;
                        let g = (color.g as f32 * alpha + bg_g * (1.0 - alpha)) as u32;
                        let b = (color.b as f32 * alpha + bg_b * (1.0 - alpha)) as u32;

                        self.buffer[idx] = (255 << 24) | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    }

    fn draw_char(&mut self, x: i32, y: i32, ch: char, color: Color, bold: bool) {
        // Try to get from cache
        let cache_key = GlyphCacheKey { ch, bold };

        // Get or create the buffer for this character
        let buffer = if let Some(cached_buffer) = self.glyph_buffer_cache.get(&cache_key) {
            // Use cached buffer (no need to shape again)
            cached_buffer
        } else {
            // Create a new buffer and cache it
            let metrics = Metrics::new(self.font_size, self.cell_height);
            let mut buffer = Buffer::new(&mut self.font_system, metrics);

            // Set buffer size to cell width to constrain text
            buffer.set_size(&mut self.font_system, self.cell_width, self.cell_height);

            let mut attrs = Attrs::new().family(cosmic_text::Family::Monospace);
            if bold {
                attrs = attrs.weight(cosmic_text::Weight::BOLD);
            }

            buffer.set_text(&mut self.font_system, &ch.to_string(), attrs, Shaping::Advanced);
            buffer.shape_until_scroll(&mut self.font_system, false);

            // Insert into cache and return reference
            self.glyph_buffer_cache.insert(cache_key, buffer);
            self.glyph_buffer_cache.get(&cache_key).unwrap()
        };

        // Cell boundaries for clipping (currently unused, kept for future optimization)
        let _cell_right = x + self.cell_width as i32;
        let _cell_bottom = y + self.cell_height as i32;

        // Render using swash
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                // Calculate glyph position with baseline offset
                // Add offset to center text vertically in the cell
                let baseline_offset = self.font_size * 1.1; // Adjust baseline position
                let glyph_x = x as f32 + glyph.x;
                let glyph_y = y as f32 + glyph.y + baseline_offset;

                let physical_glyph = glyph.physical((glyph_x, glyph_y), 1.0);

                self.swash_cache.with_pixels(
                    &mut self.font_system,
                    physical_glyph.cache_key,
                    CosmicColor::rgb(color.r, color.g, color.b),
                    |gx, gy, alpha_color| {
                        let px = physical_glyph.x + gx;
                        let py = physical_glyph.y + gy;

                        // Check bounds (disabled cell clipping for debugging)
                        if px >= 0 && px < self.width as i32
                            && py >= 0 && py < self.height as i32
                        {
                            let idx = (py * self.width as i32 + px) as usize;
                            if idx < self.buffer.len() {
                                // Blend the glyph with the background
                                let color_u32 = alpha_color.0;
                                let alpha = ((color_u32 >> 24) & 0xFF) as f32 / 255.0;
                                if alpha > 0.0 {
                                    let fg_r = ((color_u32 >> 16) & 0xFF) as f32;
                                    let fg_g = ((color_u32 >> 8) & 0xFF) as f32;
                                    let fg_b = (color_u32 & 0xFF) as f32;

                                    let bg = self.buffer[idx];
                                    let bg_r = ((bg >> 16) & 0xFF) as f32;
                                    let bg_g = ((bg >> 8) & 0xFF) as f32;
                                    let bg_b = (bg & 0xFF) as f32;

                                    let r = (fg_r * alpha + bg_r * (1.0 - alpha)) as u32;
                                    let g = (fg_g * alpha + bg_g * (1.0 - alpha)) as u32;
                                    let b = (fg_b * alpha + bg_b * (1.0 - alpha)) as u32;

                                    self.buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                                }
                            }
                        }
                    },
                );
            }
        }
    }
}

impl SoftbufferBackend {
    /// Clear the entire buffer
    fn clear_buffer(&mut self) {
        let bg_color = Color::BLACK;
        self.buffer.fill(Self::color_to_u32(bg_color));
    }

    /// Render text to buffer with offset and clipping
    fn render_text_to_buffer_with_offset(
        &mut self,
        grid: &mut Grid,
        cursor: CursorInfo,
        offset_x: i32,
        offset_y: i32,
        clip_width: u32,
        clip_height: u32,
    ) {
        let grid_cols = grid.cols();
        let grid_rows = grid.rows();

        // Always render all cells for correctness
        // Buffer is already cleared in clear() before rendering all panes
        for row in 0..grid_rows {
            for col in 0..grid_cols {
                if let Some(cell) = grid.get(col, row) {
                    let x = offset_x + (col as f32 * self.cell_width) as i32;
                    let y = offset_y + (row as f32 * self.cell_height) as i32;

                    // Clip to pane boundaries
                    if x < offset_x || x >= (offset_x + clip_width as i32) {
                        continue;
                    }
                    if y < offset_y || y >= (offset_y + clip_height as i32) {
                        continue;
                    }

                    self.render_cell_at(x, y, cell);
                }
            }
        }

        // Draw cursor
        if cursor.visible && cursor.row < grid_rows && cursor.col < grid_cols {
            let x = offset_x + (cursor.col as f32 * self.cell_width) as i32;
            let y = offset_y + (cursor.row as f32 * self.cell_height) as i32;
            let cursor_height = 2;
            let cursor_y = y + (self.cell_height * 0.8) as i32;

            // Clip cursor to pane boundaries
            if x >= offset_x && x < (offset_x + clip_width as i32) &&
               cursor_y >= offset_y && cursor_y < (offset_y + clip_height as i32) {
                self.draw_rect(x, cursor_y, self.cell_width as i32, cursor_height, Color::WHITE);
            }
        }

        // Clear dirty tracking after rendering
        grid.clear_dirty();
    }

    fn render_cell_at(&mut self, x: i32, y: i32, cell: &crate::terminal::grid::Cell) {
        // Determine colors (handle inverse)
        let (fg, bg) = if cell.attrs.inverse {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };

        // Draw background
        self.draw_rect(x, y, self.cell_width as i32, self.cell_height as i32, bg);

        // Draw character using cosmic-text
        if cell.ch != ' ' && cell.ch != '\0' {
            self.draw_char(x, y, cell.ch, fg, cell.attrs.bold);
        }
    }
}

impl RenderBackend for SoftbufferBackend {
    fn new(window: &Window, font_size: f32) -> Result<Self> {
        // SAFETY: We extend the lifetime of the window reference to 'static
        // This is safe because the window lifetime is managed by the application
        // and will outlive the backend
        let window_static: &'static Window = unsafe { std::mem::transmute(window) };

        let context = Context::new(window_static)
            .map_err(|e| TerbulatorError::rendering(format!("Failed to create softbuffer context: {}", e)))?;

        let mut surface = Surface::new(&context, window_static)
            .map_err(|e| TerbulatorError::rendering(format!("Failed to create softbuffer surface: {}", e)))?;

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        // Use fixed monospace dimensions - more reliable
        // (Previously used cosmic-text measurement, but fixed dimensions are more consistent)
        let cell_width = font_size * 0.6; // Fixed monospace width
        let cell_height = font_size * 1.3; // Line height with spacing

        let measured_width = cell_width; // For logging

        let buffer_size = (width * height) as usize;
        let buffer = vec![0u32; buffer_size];

        // Initialize surface size
        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .map_err(|e| TerbulatorError::rendering(format!("Failed to resize surface: {}", e)))?;

        log::info!(
            "SoftbufferBackend initialized: {}x{} px, cell: {}x{} px (measured: {}), font_size: {}",
            width,
            height,
            cell_width,
            cell_height,
            measured_width,
            font_size
        );

        Ok(Self {
            surface,
            font_system,
            swash_cache,
            glyph_buffer_cache: HashMap::new(),
            font_size,
            cell_width,
            cell_height,
            width,
            height,
            buffer,
        })
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.buffer.resize((width * height) as usize, 0);

            self.surface
                .resize(
                    NonZeroU32::new(width).unwrap(),
                    NonZeroU32::new(height).unwrap(),
                )
                .map_err(|e| TerbulatorError::rendering(format!("Failed to resize surface: {}", e)))?;
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.clear_buffer();
        Ok(())
    }

    fn render_frame(&mut self, grid: &mut Grid, cursor: CursorInfo) -> Result<()> {
        self.render_text_to_buffer(grid, cursor);
        Ok(())
    }

    fn render_pane(
        &mut self,
        grid: &mut Grid,
        cursor: CursorInfo,
        offset_x: i32,
        offset_y: i32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.render_text_to_buffer_with_offset(grid, cursor, offset_x, offset_y, width, height);
        Ok(())
    }

    fn draw_border(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        let border_color = Color::rgb(100, 150, 255); // Light blue border for active pane
        let border_thickness = 2;

        // Top border
        self.draw_rect(x, y, width, border_thickness, border_color);
        // Bottom border
        self.draw_rect(x, y + height - border_thickness, width, border_thickness, border_color);
        // Left border
        self.draw_rect(x, y, border_thickness, height, border_color);
        // Right border
        self.draw_rect(x + width - border_thickness, y, border_thickness, height, border_color);

        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        let mut surface_buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| TerbulatorError::rendering(format!("Failed to get surface buffer: {}", e)))?;

        // Verify buffer sizes match
        if surface_buffer.len() != self.buffer.len() {
            log::error!(
                "Buffer size mismatch: surface={}, internal={}",
                surface_buffer.len(),
                self.buffer.len()
            );
            return Err(TerbulatorError::rendering("Buffer size mismatch"));
        }

        surface_buffer.copy_from_slice(&self.buffer);

        surface_buffer
            .present()
            .map_err(|e| TerbulatorError::rendering(format!("Failed to present buffer: {}", e)))?;

        Ok(())
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Cpu
    }

    fn cell_dimensions(&self) -> (f32, f32) {
        (self.cell_width, self.cell_height)
    }

    fn render_help_overlay(&mut self, help_text: &[&str]) -> Result<()> {
        // Calculate overlay dimensions
        let max_line_width = help_text.iter().map(|s| s.len()).max().unwrap_or(0);
        let overlay_width = ((max_line_width as f32 + 4.0) * self.cell_width) as i32;
        let overlay_height = ((help_text.len() as f32 + 2.0) * self.cell_height) as i32;

        // Center the overlay
        let overlay_x = ((self.width as i32 - overlay_width) / 2).max(0);
        let overlay_y = ((self.height as i32 - overlay_height) / 2).max(0);

        // Draw semi-transparent background
        let bg_color = Color::rgb(40, 40, 60);
        self.draw_rect(overlay_x, overlay_y, overlay_width, overlay_height, bg_color);

        // Draw border
        let border_color = Color::rgb(100, 150, 255);
        let border_thickness = 3;
        self.draw_rect(overlay_x, overlay_y, overlay_width, border_thickness, border_color);
        self.draw_rect(overlay_x, overlay_y + overlay_height - border_thickness, overlay_width, border_thickness, border_color);
        self.draw_rect(overlay_x, overlay_y, border_thickness, overlay_height, border_color);
        self.draw_rect(overlay_x + overlay_width - border_thickness, overlay_y, border_thickness, overlay_height, border_color);

        // Draw text
        let text_color = Color::WHITE;
        let text_x = overlay_x + (2.0 * self.cell_width) as i32;
        let mut text_y = overlay_y + (1.0 * self.cell_height) as i32;

        for line in help_text {
            for (i, ch) in line.chars().enumerate() {
                let char_x = text_x + (i as f32 * self.cell_width) as i32;
                self.draw_char(char_x, text_y, ch, text_color, false);
            }
            text_y += self.cell_height as i32;
        }

        Ok(())
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn set_font_size(&mut self, size: f32) -> Result<()> {
        self.font_size = size;
        // Recalculate cell dimensions
        self.cell_width = size * 0.6;
        self.cell_height = size * 1.3;
        // Clear glyph cache as font size changed
        self.glyph_buffer_cache.clear();
        log::info!("Font size changed to {}, cell dimensions: {}x{}, glyph cache cleared", size, self.cell_width, self.cell_height);
        Ok(())
    }

    fn draw_selection_highlight(&mut self, col: usize, row: usize, cell_width: f32, cell_height: f32, offset_x: i32, offset_y: i32) -> Result<()> {
        let x = offset_x + (col as f32 * cell_width) as i32;
        let y = offset_y + (row as f32 * cell_height) as i32;

        // Draw semi-transparent selection highlight (light blue)
        let selection_color = Color::rgba(100, 150, 255, 128);
        self.draw_rect_blend(x, y, cell_width as i32, cell_height as i32, selection_color);

        Ok(())
    }

    fn draw_image(&mut self, image: &image::DynamicImage, x: i32, y: i32, width: u32, height: u32) -> Result<()> {
        // Resize image to target dimensions
        let resized = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();

        // Draw image pixel by pixel
        for (px, py, pixel) in rgba.enumerate_pixels() {
            let dest_x = x + px as i32;
            let dest_y = y + py as i32;

            // Check bounds
            if dest_x >= 0 && dest_x < self.width as i32 && dest_y >= 0 && dest_y < self.height as i32 {
                let idx = (dest_y * self.width as i32 + dest_x) as usize;
                if idx < self.buffer.len() {
                    // Blend pixel with background if it has alpha
                    let alpha = pixel[3] as f32 / 255.0;
                    if alpha > 0.999 {
                        // Fully opaque
                        self.buffer[idx] = 0xFF000000 | ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);
                    } else if alpha > 0.0 {
                        // Alpha blending
                        let bg = self.buffer[idx];
                        let bg_r = ((bg >> 16) & 0xFF) as f32;
                        let bg_g = ((bg >> 8) & 0xFF) as f32;
                        let bg_b = (bg & 0xFF) as f32;

                        let r = (pixel[0] as f32 * alpha + bg_r * (1.0 - alpha)) as u32;
                        let g = (pixel[1] as f32 * alpha + bg_g * (1.0 - alpha)) as u32;
                        let b = (pixel[2] as f32 * alpha + bg_b * (1.0 - alpha)) as u32;

                        self.buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }

        Ok(())
    }
}
