use crate::renderer::backend::Color;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            attrs: CellAttributes::default(),
        }
    }
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Terminal grid containing cells
pub struct Grid {
    cells: Vec<Cell>,
    cols: usize,
    rows: usize,
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
    dirty_cells: HashSet<(usize, usize)>,
    full_redraw_needed: bool,
}

impl Grid {
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> Self {
        let cells = vec![Cell::default(); cols * rows];
        Self {
            cells,
            cols,
            rows,
            scrollback: Vec::new(),
            max_scrollback,
            dirty_cells: HashSet::new(),
            full_redraw_needed: true,
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.cells.resize(cols * rows, Cell::default());
        self.full_redraw_needed = true;
        self.dirty_cells.clear();
    }

    pub fn get(&self, col: usize, row: usize) -> Option<&Cell> {
        if col < self.cols && row < self.rows {
            Some(&self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        if col < self.cols && row < self.rows {
            Some(&mut self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    pub fn set(&mut self, col: usize, row: usize, cell: Cell) {
        if let Some(c) = self.get_mut(col, row) {
            if *c != cell {
                *c = cell;
                self.dirty_cells.insert((col, row));
            }
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
        self.full_redraw_needed = true;
        self.dirty_cells.clear();
    }

    pub fn clear_row(&mut self, row: usize) {
        if row < self.rows {
            let start = row * self.cols;
            let end = start + self.cols;
            for cell in &mut self.cells[start..end] {
                cell.reset();
            }
            // Mark entire row as dirty
            for col in 0..self.cols {
                self.dirty_cells.insert((col, row));
            }
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        if lines == 0 || lines >= self.rows {
            return;
        }

        // Save top lines to scrollback
        for i in 0..lines {
            let start = i * self.cols;
            let end = start + self.cols;
            let line = self.cells[start..end].to_vec();
            self.scrollback.push(line);

            // Limit scrollback size
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
        }

        // Shift cells up
        let shift_amount = lines * self.cols;
        self.cells.copy_within(shift_amount.., 0);

        // Clear bottom lines
        let clear_start = (self.rows - lines) * self.cols;
        for cell in &mut self.cells[clear_start..] {
            cell.reset();
        }

        // Scroll affects entire screen
        self.full_redraw_needed = true;
        self.dirty_cells.clear();
    }

    pub fn scroll_down(&mut self, lines: usize) {
        if lines == 0 || lines >= self.rows {
            return;
        }

        // Shift cells down
        let shift_amount = lines * self.cols;
        self.cells.copy_within(..self.cols * (self.rows - lines), shift_amount);

        // Clear top lines
        let clear_end = lines * self.cols;
        for cell in &mut self.cells[..clear_end] {
            cell.reset();
        }

        // Scroll affects entire screen
        self.full_redraw_needed = true;
        self.dirty_cells.clear();
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[Cell]> {
        self.cells.chunks(self.cols)
    }

    pub fn get_row(&self, row: usize) -> Option<&[Cell]> {
        if row < self.rows {
            let start = row * self.cols;
            let end = start + self.cols;
            Some(&self.cells[start..end])
        } else {
            None
        }
    }

    /// Check if full redraw is needed
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw_needed
    }

    /// Get dirty cells (changed since last frame)
    pub fn dirty_cells(&self) -> &HashSet<(usize, usize)> {
        &self.dirty_cells
    }

    /// Clear dirty tracking (called after rendering)
    pub fn clear_dirty(&mut self) {
        self.dirty_cells.clear();
        self.full_redraw_needed = false;
    }
}
