use crate::renderer::backend::Color;
use crate::terminal::grid::{Cell, CellAttributes, Grid};
use crate::terminal::image::{KittyImageParser, SixelImageParser, TerminalImage};
use vte::{Params, Perform};

pub struct TerminalEmulator {
    grid: Grid,
    cursor_col: usize,
    cursor_row: usize,
    cursor_visible: bool,
    current_fg: Color,
    current_bg: Color,
    current_attrs: CellAttributes,
    saved_cursor: Option<(usize, usize)>,
    parser: vte::Parser,
    kitty_parser: KittyImageParser,
    sixel_parser: SixelImageParser,
    images: Vec<TerminalImage>,
}

impl TerminalEmulator {
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Self {
        Self {
            grid: Grid::new(cols, rows, scrollback),
            cursor_col: 0,
            cursor_row: 0,
            cursor_visible: true,
            current_fg: Color::WHITE,
            current_bg: Color::BLACK,
            current_attrs: CellAttributes::default(),
            saved_cursor: None,
            parser: vte::Parser::new(),
            kitty_parser: KittyImageParser::new(),
            sixel_parser: SixelImageParser::new(),
            images: Vec::new(),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.grid.resize(cols, rows);
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
    }

    pub fn process_byte(&mut self, byte: u8) {
        // Temporarily take the parser to avoid borrowing issues
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());
        parser.advance(self, byte);
        self.parser = parser;
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        // Process all bytes with parser to avoid repeated moves
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());
        for &byte in bytes {
            parser.advance(self, byte);

            // Also try to parse images
            if let Some(image) = self.kitty_parser.process_byte(byte) {
                self.add_image(image);
            }
            if let Some(image) = self.sixel_parser.process_byte(byte) {
                self.add_image(image);
            }
        }
        self.parser = parser;
    }

    fn add_image(&mut self, image: image::DynamicImage) {
        // Calculate image dimensions in cells
        let cell_width = 10.0; // Approximate cell width in pixels (will be refined later)
        let cell_height = 20.0; // Approximate cell height in pixels

        let width_cells = ((image.width() as f32 / cell_width).ceil() as usize).max(1);
        let height_cells = ((image.height() as f32 / cell_height).ceil() as usize).max(1);

        let terminal_image = TerminalImage::new(
            image,
            self.cursor_row,
            self.cursor_col,
            width_cells,
            height_cells,
        );

        log::info!(
            "Added image at ({}, {}), size: {}x{} cells",
            self.cursor_col,
            self.cursor_row,
            width_cells,
            height_cells
        );

        self.images.push(terminal_image);

        // Move cursor after the image
        self.cursor_row += height_cells;
        if self.cursor_row >= self.grid.rows() {
            self.cursor_row = self.grid.rows() - 1;
        }
    }

    pub fn images(&self) -> &[TerminalImage] {
        &self.images
    }

    fn write_char(&mut self, ch: char) {
        if self.cursor_col >= self.grid.cols() {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.grid.rows() {
                self.grid.scroll_up(1);
                self.cursor_row = self.grid.rows() - 1;
            }
        }

        let mut cell = Cell::new(ch);
        cell.fg = self.current_fg;
        cell.bg = self.current_bg;
        cell.attrs = self.current_attrs;

        self.grid.set(self.cursor_col, self.cursor_row, cell);
        self.cursor_col += 1;
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn line_feed(&mut self) {
        self.cursor_row += 1;
        if self.cursor_row >= self.grid.rows() {
            self.grid.scroll_up(1);
            self.cursor_row = self.grid.rows() - 1;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn tab(&mut self) {
        // Move to next tab stop (every 8 columns)
        self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
        if self.cursor_col >= self.grid.cols() {
            self.cursor_col = self.grid.cols() - 1;
        }
    }

    fn set_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            // Reset all attributes
            self.current_fg = Color::WHITE;
            self.current_bg = Color::BLACK;
            self.current_attrs = CellAttributes::default();
            return;
        }

        let mut iter = params.iter();
        while let Some(param) = iter.next() {
            let n = param[0];
            match n {
                0 => {
                    // Reset
                    self.current_fg = Color::WHITE;
                    self.current_bg = Color::BLACK;
                    self.current_attrs = CellAttributes::default();
                }
                1 => self.current_attrs.bold = true,
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = true,
                7 => self.current_attrs.inverse = true,
                22 => self.current_attrs.bold = false,
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = false,
                27 => self.current_attrs.inverse = false,
                // Foreground colors (30-37, 90-97)
                30..=37 => self.current_fg = Color::from_ansi_256((n - 30) as u8),
                90..=97 => self.current_fg = Color::from_ansi_256((n - 90 + 8) as u8),
                // Background colors (40-47, 100-107)
                40..=47 => self.current_bg = Color::from_ansi_256((n - 40) as u8),
                100..=107 => self.current_bg = Color::from_ansi_256((n - 100 + 8) as u8),
                // 256-color mode
                38 => {
                    if let Some(next) = iter.next() {
                        if next[0] == 5 {
                            if let Some(color) = iter.next() {
                                self.current_fg = Color::from_ansi_256(color[0] as u8);
                            }
                        }
                    }
                }
                48 => {
                    if let Some(next) = iter.next() {
                        if next[0] == 5 {
                            if let Some(color) = iter.next() {
                                self.current_bg = Color::from_ansi_256(color[0] as u8);
                            }
                        }
                    }
                }
                39 => self.current_fg = Color::WHITE, // Default foreground
                49 => self.current_bg = Color::BLACK, // Default background
                _ => {}
            }
        }
    }
}

impl Perform for TerminalEmulator {
    fn print(&mut self, c: char) {
        self.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.line_feed(),
            b'\r' => self.carriage_return(),
            b'\x08' => self.backspace(),
            b'\t' => self.tab(),
            b'\x07' => {} // Bell - ignore for now
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'H' | 'f' => {
                // Cursor position
                let row = if params.is_empty() {
                    1
                } else {
                    params.iter().next().unwrap()[0].max(1)
                };
                let col = if params.len() < 2 {
                    1
                } else {
                    params.iter().nth(1).unwrap()[0].max(1)
                };
                self.cursor_row = (row as usize - 1).min(self.grid.rows() - 1);
                self.cursor_col = (col as usize - 1).min(self.grid.cols() - 1);
            }
            'A' => {
                // Cursor up
                let n = if params.is_empty() { 1 } else { params.iter().next().unwrap()[0].max(1) };
                self.cursor_row = self.cursor_row.saturating_sub(n as usize);
            }
            'B' => {
                // Cursor down
                let n = if params.is_empty() { 1 } else { params.iter().next().unwrap()[0].max(1) };
                self.cursor_row = (self.cursor_row + n as usize).min(self.grid.rows() - 1);
            }
            'C' => {
                // Cursor forward
                let n = if params.is_empty() { 1 } else { params.iter().next().unwrap()[0].max(1) };
                self.cursor_col = (self.cursor_col + n as usize).min(self.grid.cols() - 1);
            }
            'D' => {
                // Cursor backward
                let n = if params.is_empty() { 1 } else { params.iter().next().unwrap()[0].max(1) };
                self.cursor_col = self.cursor_col.saturating_sub(n as usize);
            }
            'J' => {
                // Erase in display
                let n = if params.is_empty() { 0 } else { params.iter().next().unwrap()[0] };
                match n {
                    0 => {
                        // Clear from cursor to end of screen
                        for col in self.cursor_col..self.grid.cols() {
                            if let Some(cell) = self.grid.get_mut(col, self.cursor_row) {
                                cell.reset();
                            }
                        }
                        for row in (self.cursor_row + 1)..self.grid.rows() {
                            self.grid.clear_row(row);
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of screen
                        for row in 0..self.cursor_row {
                            self.grid.clear_row(row);
                        }
                        for col in 0..=self.cursor_col {
                            if let Some(cell) = self.grid.get_mut(col, self.cursor_row) {
                                cell.reset();
                            }
                        }
                    }
                    2 | 3 => {
                        // Clear entire screen
                        self.grid.clear();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in line
                let n = if params.is_empty() { 0 } else { params.iter().next().unwrap()[0] };
                match n {
                    0 => {
                        // Clear from cursor to end of line
                        for col in self.cursor_col..self.grid.cols() {
                            if let Some(cell) = self.grid.get_mut(col, self.cursor_row) {
                                cell.reset();
                            }
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of line
                        for col in 0..=self.cursor_col {
                            if let Some(cell) = self.grid.get_mut(col, self.cursor_row) {
                                cell.reset();
                            }
                        }
                    }
                    2 => {
                        // Clear entire line
                        self.grid.clear_row(self.cursor_row);
                    }
                    _ => {}
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.set_sgr(params);
            }
            's' => {
                // Save cursor position
                self.saved_cursor = Some((self.cursor_col, self.cursor_row));
            }
            'u' => {
                // Restore cursor position
                if let Some((col, row)) = self.saved_cursor {
                    self.cursor_col = col;
                    self.cursor_row = row;
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'c' => {
                // RIS - Reset to Initial State
                log::debug!("Reset to initial state (RIS)");
                self.grid.clear();
                self.cursor_col = 0;
                self.cursor_row = 0;
                self.cursor_visible = true;
                self.current_fg = Color::WHITE;
                self.current_bg = Color::BLACK;
                self.current_attrs = CellAttributes::default();
                self.saved_cursor = None;
            }
            _ => {
                log::trace!("Unhandled ESC dispatch: byte={}", byte);
            }
        }
    }
}
