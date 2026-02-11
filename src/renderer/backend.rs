use crate::terminal::Grid;
use crate::utils::Result;
use winit::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    Gpu,
    Cpu,
}

/// Common cell representation for rendering
#[derive(Debug, Clone, Copy)]
pub struct RenderCell {
    pub ch: char,
    pub fg_color: Color,
    pub bg_color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(r, g, b, a)
    }

    pub fn from_ansi_256(index: u8) -> Self {
        // ANSI 256 color palette
        match index {
            // 16 basic colors
            0 => Self::rgb(0, 0, 0),       // Black
            1 => Self::rgb(205, 0, 0),     // Red
            2 => Self::rgb(0, 205, 0),     // Green
            3 => Self::rgb(205, 205, 0),   // Yellow
            4 => Self::rgb(0, 0, 238),     // Blue
            5 => Self::rgb(205, 0, 205),   // Magenta
            6 => Self::rgb(0, 205, 205),   // Cyan
            7 => Self::rgb(229, 229, 229), // White
            8 => Self::rgb(127, 127, 127), // Bright Black
            9 => Self::rgb(255, 0, 0),     // Bright Red
            10 => Self::rgb(0, 255, 0),    // Bright Green
            11 => Self::rgb(255, 255, 0),  // Bright Yellow
            12 => Self::rgb(92, 92, 255),  // Bright Blue
            13 => Self::rgb(255, 0, 255),  // Bright Magenta
            14 => Self::rgb(0, 255, 255),  // Bright Cyan
            15 => Self::rgb(255, 255, 255), // Bright White

            // 216 color cube (16-231)
            16..=231 => {
                let idx = index - 16;
                let r = (idx / 36) * 51;
                let g = ((idx % 36) / 6) * 51;
                let b = (idx % 6) * 51;
                Self::rgb(r, g, b)
            }

            // 24 grayscale colors (232-255)
            232..=255 => {
                let gray = 8 + (index - 232) * 10;
                Self::rgb(gray, gray, gray)
            }
        }
    }

    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
}

/// Cursor position and style
#[derive(Debug, Clone, Copy)]
pub struct CursorInfo {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
}

/// Abstract rendering backend trait
pub trait RenderBackend {
    /// Initialize the backend
    fn new(window: &Window, font_size: f32) -> Result<Self>
    where
        Self: Sized;

    /// Handle window resize
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;

    /// Clear the rendering buffer
    fn clear(&mut self) -> Result<()>;

    /// Render a frame with the given grid
    fn render_frame(&mut self, grid: &mut Grid, cursor: CursorInfo) -> Result<()>;

    /// Render a pane at a specific offset with clipping
    fn render_pane(
        &mut self,
        grid: &mut Grid,
        cursor: CursorInfo,
        offset_x: i32,
        offset_y: i32,
        width: u32,
        height: u32,
    ) -> Result<()>;

    /// Draw a border around a rectangular region
    fn draw_border(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()>;

    /// Draw selection highlight for a cell
    fn draw_selection_highlight(&mut self, col: usize, row: usize, cell_width: f32, cell_height: f32, offset_x: i32, offset_y: i32) -> Result<()>;

    /// Draw an image at the specified position
    fn draw_image(&mut self, image: &image::DynamicImage, x: i32, y: i32, width: u32, height: u32) -> Result<()>;

    /// Present the rendered frame to the window
    fn present(&mut self) -> Result<()>;

    /// Get the backend type
    fn backend_type(&self) -> BackendType;

    /// Get cell dimensions in pixels
    fn cell_dimensions(&self) -> (f32, f32);

    /// Render help overlay on top of current frame
    fn render_help_overlay(&mut self, help_text: &[&str]) -> Result<()>;

    /// Get current font size
    fn font_size(&self) -> f32;

    /// Set font size and recalculate cell dimensions
    fn set_font_size(&mut self, size: f32) -> Result<()>;
}
