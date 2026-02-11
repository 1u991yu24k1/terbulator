use crate::terminal::Grid;

/// Text selection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Start position (col, row)
    pub start: (usize, usize),
    /// End position (col, row)
    pub end: (usize, usize),
    /// Whether selection is active
    pub active: bool,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self {
            start: (0, 0),
            end: (0, 0),
            active: false,
        }
    }

    /// Start a new selection at the given position (not active until drag)
    pub fn start_at(&mut self, col: usize, row: usize) {
        self.start = (col, row);
        self.end = (col, row);
        self.active = false; // Don't activate until actual drag occurs
    }

    /// Update the end position of the selection (activates on first drag)
    pub fn update_end(&mut self, col: usize, row: usize) {
        self.end = (col, row);
        // Activate selection only if position has changed from start
        if self.start != self.end {
            self.active = true;
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.active = false;
        self.start = (0, 0);
        self.end = (0, 0);
    }

    /// Check if a cell is within the selection
    pub fn contains(&self, col: usize, row: usize) -> bool {
        if !self.active {
            return false;
        }

        let (start_col, start_row) = self.normalized_start();
        let (end_col, end_row) = self.normalized_end();

        // Check if row is within range
        if row < start_row || row > end_row {
            return false;
        }

        // Single row selection
        if start_row == end_row {
            return col >= start_col && col <= end_col;
        }

        // Multi-row selection
        if row == start_row {
            col >= start_col
        } else if row == end_row {
            col <= end_col
        } else {
            true
        }
    }

    /// Get the normalized start position (always the earlier position)
    fn normalized_start(&self) -> (usize, usize) {
        if self.start.1 < self.end.1 || (self.start.1 == self.end.1 && self.start.0 <= self.end.0) {
            self.start
        } else {
            self.end
        }
    }

    /// Get the normalized end position (always the later position)
    fn normalized_end(&self) -> (usize, usize) {
        if self.start.1 < self.end.1 || (self.start.1 == self.end.1 && self.start.0 <= self.end.0) {
            self.end
        } else {
            self.start
        }
    }

    /// Extract selected text from the grid
    pub fn get_text(&self, grid: &Grid) -> String {
        if !self.active {
            return String::new();
        }

        let (start_col, start_row) = self.normalized_start();
        let (end_col, end_row) = self.normalized_end();

        let mut text = String::new();

        for row in start_row..=end_row {
            if row >= grid.rows() {
                break;
            }

            let row_start = if row == start_row { start_col } else { 0 };
            let row_end = if row == end_row { end_col } else { grid.cols() - 1 };

            for col in row_start..=row_end {
                if col >= grid.cols() {
                    break;
                }

                if let Some(cell) = grid.get(col, row) {
                    if cell.ch != '\0' {
                        text.push(cell.ch);
                    }
                }
            }

            // Add newline if not the last row
            if row < end_row {
                text.push('\n');
            }
        }

        text
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}
