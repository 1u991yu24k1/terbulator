mod kitty;
mod sixel;

pub use kitty::KittyImageParser;
pub use sixel::SixelImageParser;

use image::DynamicImage;

/// Image data stored in the terminal grid
#[derive(Debug, Clone)]
pub struct TerminalImage {
    /// The decoded image
    pub image: DynamicImage,
    /// Row where the image starts
    pub row: usize,
    /// Column where the image starts
    pub col: usize,
    /// Width in cells
    pub width_cells: usize,
    /// Height in cells
    pub height_cells: usize,
}

impl TerminalImage {
    pub fn new(image: DynamicImage, row: usize, col: usize, width_cells: usize, height_cells: usize) -> Self {
        Self {
            image,
            row,
            col,
            width_cells,
            height_cells,
        }
    }

    /// Get the image width in pixels
    pub fn width_pixels(&self) -> u32 {
        self.image.width()
    }

    /// Get the image height in pixels
    pub fn height_pixels(&self) -> u32 {
        self.image.height()
    }
}
