use crate::clipboard::{ClipboardManager, Selection};
use crate::config::Config;
use crate::input::{KeyboardHandler, ShortcutHandler, ShortcutAction};
use crate::pane::{PaneManager, Rect};
use crate::renderer::backend::{BackendType, CursorInfo, RenderBackend};
use crate::renderer::softbuffer_backend::SoftbufferBackend;
use crate::utils::Result;
use std::time::{Duration, Instant};
use winit::window::Window;

/// Central application state
pub struct App {
    pub config: Config,
    pane_manager: PaneManager,
    renderer: Box<dyn RenderBackend>,
    keyboard: KeyboardHandler,
    shortcuts: ShortcutHandler,
    clipboard_manager: ClipboardManager,
    selection: Selection,
    selecting: bool, // Track if user is currently selecting text
    mark_mode: bool, // Track if mark mode is active (keyboard-based selection)
    mark_cursor: Option<(usize, usize)>, // Mark mode cursor position (col, row)
    ime_enabled: bool, // Track if IME is enabled
    last_cursor_blink: Instant,
    cursor_visible: bool,
    help_visible: bool,
    window_width: u32,
    window_height: u32,
    dragging_border: bool,
}

impl App {
    pub fn new(config: Config, window: &Window) -> Result<Self> {
        let cols = config.terminal.cols;
        let rows = config.terminal.rows;
        let font_size = config.terminal.font_size;
        let scrollback = config.terminal.scrollback;
        let shell = config.terminal.shell.clone();

        // Create pane manager with initial pane
        let pane_manager = PaneManager::new(cols, rows, scrollback, shell)?;

        // Create renderer based on config
        let renderer: Box<dyn RenderBackend> = match config.renderer.backend.as_str() {
            "cpu" => {
                log::info!("Using CPU rendering backend (softbuffer)");
                Box::new(SoftbufferBackend::new(window, font_size)?)
            }
            "gpu" => {
                log::warn!("GPU backend not yet fully implemented, falling back to CPU");
                Box::new(SoftbufferBackend::new(window, font_size)?)
            }
            "auto" | _ => {
                log::info!("Auto-selecting rendering backend: using CPU (softbuffer)");
                Box::new(SoftbufferBackend::new(window, font_size)?)
            }
        };

        let keyboard = KeyboardHandler::new();
        let shortcuts = ShortcutHandler::new();
        let clipboard_manager = ClipboardManager::new()?;
        let selection = Selection::new();

        let size = window.inner_size();

        log::info!("App initialized successfully");
        log::info!(
            "Terminal size: {}x{}, Cell size: {:?}",
            cols,
            rows,
            renderer.cell_dimensions()
        );

        let mut app = Self {
            config: config.clone(),
            pane_manager,
            renderer,
            keyboard,
            shortcuts,
            clipboard_manager,
            selection,
            selecting: false,
            mark_mode: false,
            mark_cursor: None,
            ime_enabled: false,
            last_cursor_blink: Instant::now(),
            cursor_visible: true,
            help_visible: false,
            window_width: size.width,
            window_height: size.height,
            dragging_border: false,
        };

        // Initialize startup panes according to config
        app.initialize_startup_panes(&config)?;

        Ok(app)
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.window_width = width;
        self.window_height = height;

        let window_rect = Rect::new(0, 0, width, height);
        let (cell_width, cell_height) = self.renderer.cell_dimensions();

        // Resize all panes based on new window size
        self.pane_manager.resize_all_panes(window_rect, cell_width, cell_height)?;

        self.renderer.resize(width, height)?;
        Ok(())
    }

    pub fn handle_keyboard_input(&mut self, key: &winit::keyboard::PhysicalKey, modifiers: winit::keyboard::ModifiersState) -> Result<()> {
        // Check for F1 (help toggle)
        if let winit::keyboard::PhysicalKey::Code(key_code) = key {
            if *key_code == winit::keyboard::KeyCode::F1 {
                self.help_visible = !self.help_visible;
                log::info!("Help display toggled: {}", self.help_visible);
                return Ok(());
            }

            // ESC key closes help if visible
            if *key_code == winit::keyboard::KeyCode::Escape && self.help_visible {
                self.help_visible = false;
                log::info!("Help display closed");
                return Ok(());
            }
        }

        // If help is visible, don't process other keys
        if self.help_visible {
            return Ok(());
        }

        // Check for mark mode navigation (if mark mode is active)
        if let winit::keyboard::PhysicalKey::Code(key_code) = key {
            if self.handle_mark_mode_navigation(*key_code) {
                // Mark mode handled the key
                return Ok(());
            }
        }

        // Check for shortcuts
        if let winit::keyboard::PhysicalKey::Code(key_code) = key {
            log::trace!("Key pressed: {:?}, modifiers: ctrl={}, shift={}", key_code, modifiers.control_key(), modifiers.shift_key());
            if let Some(action) = self.shortcuts.match_shortcut(*key_code, modifiers) {
                log::info!("Shortcut detected: {:?}", action);
                return self.handle_shortcut_action(action);
            }
        }

        // Regular keyboard input
        if let Some(bytes) = self.keyboard.handle_key(key) {
            log::debug!("Keyboard input: {:?} -> {} bytes", key, bytes.len());
            self.pane_manager.write_input(&bytes)?;
            log::debug!("Written to pane(s) successfully");
        } else {
            log::debug!("Keyboard input ignored: {:?}", key);
        }
        Ok(())
    }

    fn handle_shortcut_action(&mut self, action: ShortcutAction) -> Result<()> {
        use crate::pane::SplitDirection;

        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
        let (cell_width, cell_height) = self.renderer.cell_dimensions();

        match action {
            ShortcutAction::SplitHorizontal => {
                match self.pane_manager.split_active_pane(SplitDirection::Horizontal, window_rect, cell_width, cell_height) {
                    Ok(new_id) => log::info!("Split pane horizontally, created pane {}", new_id),
                    Err(e) => log::error!("Failed to split pane horizontally: {}", e),
                }
            }
            ShortcutAction::SplitVertical => {
                match self.pane_manager.split_active_pane(SplitDirection::Vertical, window_rect, cell_width, cell_height) {
                    Ok(new_id) => log::info!("Split pane vertically, created pane {}", new_id),
                    Err(e) => log::error!("Failed to split pane vertically: {}", e),
                }
            }
            ShortcutAction::ClosePane => {
                match self.pane_manager.close_active_pane(window_rect, cell_width, cell_height) {
                    Ok(closed) => {
                        if closed {
                            log::info!("Closed active pane");
                        } else {
                            log::info!("Cannot close last pane");
                        }
                    }
                    Err(e) => log::error!("Failed to close pane: {}", e),
                }
            }
            ShortcutAction::MoveFocusLeft => {
                if self.pane_manager.focus_left(window_rect) {
                    log::info!("Moved focus left to pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::MoveFocusDown => {
                if self.pane_manager.focus_down(window_rect) {
                    log::info!("Moved focus down to pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::MoveFocusUp => {
                if self.pane_manager.focus_up(window_rect) {
                    log::info!("Moved focus up to pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::MoveFocusRight => {
                if self.pane_manager.focus_right(window_rect) {
                    log::info!("Moved focus right to pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::MoveFocusNext => {
                if self.pane_manager.focus_next() {
                    log::info!("Moved focus to next pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::MoveFocusPrev => {
                if self.pane_manager.focus_prev() {
                    log::info!("Moved focus to previous pane {}", self.pane_manager.active_pane_id());
                }
            }
            ShortcutAction::ToggleBroadcast => {
                self.pane_manager.toggle_broadcast();
                // Window title will be updated in main event loop
            }
            ShortcutAction::IncreaseFontSize => {
                self.change_font_size(1.0)?;
            }
            ShortcutAction::DecreaseFontSize => {
                self.change_font_size(-1.0)?;
            }
            ShortcutAction::Copy => {
                self.handle_copy()?;
            }
            ShortcutAction::Paste => {
                self.handle_paste()?;
            }
            ShortcutAction::ToggleMarkMode => {
                self.toggle_mark_mode();
            }
        }

        Ok(())
    }

    pub fn update_modifiers(&mut self, modifiers: winit::keyboard::ModifiersState) {
        self.keyboard.update_modifiers(modifiers);
    }

    /// Process PTY output from all panes
    /// Returns: (has_output, should_exit)
    /// - has_output: whether any pane had output
    /// - should_exit: whether all panes have exited and app should exit
    pub fn process_pty_output(&mut self) -> Result<(bool, bool)> {
        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
        let (cell_width, cell_height) = self.renderer.cell_dimensions();
        self.pane_manager.process_all_pty_output(window_rect, cell_width, cell_height)
    }

    pub fn update_cursor_blink(&mut self) {
        // Blink cursor every 500ms
        if self.last_cursor_blink.elapsed() > Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_blink = Instant::now();
        }
    }

    pub fn render(&mut self) -> Result<()> {
        self.update_cursor_blink();

        // Check if any pane needs redraw
        let mut any_pane_needs_redraw = false;
        let active_pane_id = self.pane_manager.active_pane_id();

        // Active pane with cursor blink or selection always needs redraw
        let active_needs_redraw = self.selection.active || true; // Always redraw for cursor blink

        for pane_id in 0..100 { // Arbitrary upper limit
            if let Some(pane) = self.pane_manager.pane(pane_id) {
                if pane.needs_redraw() || (pane_id == active_pane_id && active_needs_redraw) {
                    any_pane_needs_redraw = true;
                    break;
                }
            } else {
                break;
            }
        }

        // Only render if something changed
        if !any_pane_needs_redraw {
            return Ok(());
        }

        // Clear the buffer before rendering
        self.renderer.clear()?;

        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
        let pane_rects = self.pane_manager.layout().calculate_rects(window_rect);

        // Render ALL panes (to avoid black areas from cleared buffer)
        // But only if at least one pane needs redraw
        for (pane_id, pane_rect) in &pane_rects {
            // Get cursor info (immutable borrow)
            let (cursor_info, is_active) = if let Some(pane) = self.pane_manager.pane(*pane_id) {
                let (col, row) = pane.terminal().cursor_position();
                let is_active = *pane_id == active_pane_id;
                let cursor = CursorInfo {
                    col,
                    row,
                    visible: is_active && self.cursor_visible && pane.terminal().cursor_visible(),
                };
                (cursor, is_active)
            } else {
                continue;
            };

            // Now get mutable reference to render
            if let Some(pane) = self.pane_manager.pane_mut(*pane_id) {
                let offset_x = pane_rect.x;
                let offset_y = pane_rect.y;

                // Render the pane with offset
                self.renderer.render_pane(
                    pane.terminal_mut().grid_mut(),
                    cursor_info,
                    offset_x as i32,
                    offset_y as i32,
                    pane_rect.width,
                    pane_rect.height,
                )?;

                // Clear the redraw flag after rendering
                pane.clear_redraw_flag();

                // Draw border around active pane
                if is_active && pane_rects.len() > 1 {
                    self.renderer.draw_border(
                        offset_x as i32,
                        offset_y as i32,
                        pane_rect.width as i32,
                        pane_rect.height as i32,
                    )?;
                }

                // Draw selection highlight if active
                if is_active && self.selection.active {
                    let (cell_width, cell_height) = self.renderer.cell_dimensions();
                    let grid = pane.terminal().grid();

                    // Render selection highlight for all selected cells
                    for row in 0..grid.rows() {
                        for col in 0..grid.cols() {
                            if self.selection.contains(col, row) {
                                self.renderer.draw_selection_highlight(
                                    col,
                                    row,
                                    cell_width,
                                    cell_height,
                                    offset_x as i32,
                                    offset_y as i32,
                                )?;
                            }
                        }
                    }
                }

                // Draw images
                let (cell_width, cell_height) = self.renderer.cell_dimensions();
                for image in pane.terminal().images() {
                    let img_x = offset_x as i32 + (image.col as f32 * cell_width) as i32;
                    let img_y = offset_y as i32 + (image.row as f32 * cell_height) as i32;
                    let img_width = (image.width_cells as f32 * cell_width) as u32;
                    let img_height = (image.height_cells as f32 * cell_height) as u32;

                    self.renderer.draw_image(
                        &image.image,
                        img_x,
                        img_y,
                        img_width,
                        img_height,
                    )?;
                }
            }
        }

        // Draw help overlay if visible
        if self.help_visible {
            self.render_help_overlay()?;
        }

        self.renderer.present()?;

        Ok(())
    }

    fn initialize_startup_panes(&mut self, config: &Config) -> Result<()> {
        let num_panes = config.startup.panes;
        let layout = &config.startup.layout;
        let split_ratio = config.startup.split_ratio;
        let vertical_ratio = config.startup.vertical_ratio;

        if num_panes <= 1 {
            // Single pane, already created
            log::info!("Starting with single pane");
            return Ok(());
        }

        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
        let (cell_width, cell_height) = self.renderer.cell_dimensions();

        match layout.as_str() {
            "horizontal" if num_panes == 2 => {
                // Split horizontally once with specified ratio
                log::info!("Creating 2 panes with horizontal split ({}:{})", (split_ratio * 10.0) as usize, ((1.0 - split_ratio) * 10.0) as usize);
                self.pane_manager.split_active_pane_with_ratio(
                    crate::pane::SplitDirection::Horizontal,
                    window_rect,
                    cell_width,
                    cell_height,
                    split_ratio,
                )?;
            }
            "vertical" if num_panes == 2 => {
                // Split vertically once with specified ratio
                log::info!("Creating 2 panes with vertical split ({}:{})", (vertical_ratio * 10.0) as usize, ((1.0 - vertical_ratio) * 10.0) as usize);
                self.pane_manager.split_active_pane_with_ratio(
                    crate::pane::SplitDirection::Vertical,
                    window_rect,
                    cell_width,
                    cell_height,
                    vertical_ratio,
                )?;
            }
            "grid" if num_panes == 4 => {
                // Create 4-pane grid with specified ratios
                log::info!("Creating 4-pane grid with horizontal {}:{} and vertical {}:{} ratios",
                    (split_ratio * 10.0) as usize, ((1.0 - split_ratio) * 10.0) as usize,
                    (vertical_ratio * 10.0) as usize, ((1.0 - vertical_ratio) * 10.0) as usize);

                // First horizontal split (top and bottom) with horizontal ratio
                self.pane_manager.split_active_pane_with_ratio(
                    crate::pane::SplitDirection::Horizontal,
                    window_rect,
                    cell_width,
                    cell_height,
                    split_ratio,
                )?;

                // Split top pane vertically with vertical ratio
                self.pane_manager.set_active_pane(0);
                self.pane_manager.split_active_pane_with_ratio(
                    crate::pane::SplitDirection::Vertical,
                    window_rect,
                    cell_width,
                    cell_height,
                    vertical_ratio,
                )?;

                // Split bottom pane vertically with vertical ratio
                self.pane_manager.set_active_pane(1);
                self.pane_manager.split_active_pane_with_ratio(
                    crate::pane::SplitDirection::Vertical,
                    window_rect,
                    cell_width,
                    cell_height,
                    vertical_ratio,
                )?;

                // Set active pane to first one
                self.pane_manager.set_active_pane(0);
            }
            _ => {
                log::warn!("Unsupported startup layout: {} with {} panes, using single pane", layout, num_panes);
            }
        }

        Ok(())
    }

    fn render_help_overlay(&mut self) -> Result<()> {
        // Render help text in the center of the screen
        let help_text = vec![
            "=== Terbulator Help ===",
            "",
            "Pane Management:",
            "  Ctrl+Shift+S    Split Horizontal",
            "  Ctrl+Shift+V    Split Vertical",
            "  Ctrl+Shift+W    Close Pane",
            "",
            "Focus Movement:",
            "  Ctrl+Shift+H    Focus Left",
            "  Ctrl+Shift+J    Focus Down",
            "  Ctrl+Shift+K    Focus Up",
            "  Ctrl+Shift+L    Focus Right",
            "  Ctrl+Shift+N    Focus Next",
            "  Ctrl+Shift+P    Focus Previous",
            "",
            "Font Size:",
            "  Ctrl++          Increase Font Size",
            "  Ctrl+-          Decrease Font Size",
            "",
            "Clipboard:",
            "  Mouse Drag      Select Text",
            "  Ctrl+Shift+C    Copy Selection",
            "  Ctrl+V          Paste",
            "",
            "Broadcast Mode:",
            "  Ctrl+Shift+B    Toggle Broadcast",
            "                  (Shows 'Broadcasting' in title)",
            "",
            "Other:",
            "  F1              Toggle Help",
            "  ESC             Close Help",
            "",
            "Press F1 or ESC to close this help",
        ];

        self.renderer.render_help_overlay(&help_text)?;
        Ok(())
    }

    pub fn grid_info(&self) -> (usize, usize) {
        if let Some(pane) = self.pane_manager.active_pane() {
            (pane.terminal().grid().cols(), pane.terminal().grid().rows())
        } else {
            (80, 24) // Default fallback
        }
    }

    pub fn backend_type(&self) -> BackendType {
        self.renderer.backend_type()
    }

    /// Handle mouse button press
    pub fn handle_mouse_press(&mut self, x: f64, y: f64) -> Result<()> {
        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
        let x_u32 = x as u32;
        let y_u32 = y as u32;

        // Check if clicking on a border
        if self.pane_manager.is_near_border(x_u32, y_u32, window_rect) {
            log::info!("Started dragging border at ({}, {})", x, y);
            self.dragging_border = true;
            return Ok(());
        }

        // Check which pane was clicked
        let rects = self.pane_manager.layout().calculate_rects(window_rect);
        for (pane_id, rect) in rects {
            if x >= rect.x as f64 && x < (rect.x + rect.width) as f64 &&
               y >= rect.y as f64 && y < (rect.y + rect.height) as f64 {
                log::info!("Mouse clicked on pane {} at ({}, {})", pane_id, x, y);
                self.pane_manager.set_active_pane(pane_id);

                // Start text selection
                let (cell_width, cell_height) = self.renderer.cell_dimensions();
                let col = ((x - rect.x as f64) / cell_width as f64) as usize;
                let row = ((y - rect.y as f64) / cell_height as f64) as usize;

                self.selection.start_at(col, row);
                self.selecting = true;
                log::debug!("Started selection at ({}, {})", col, row);

                return Ok(());
            }
        }

        log::debug!("Mouse click at ({}, {}) outside any pane", x, y);
        Ok(())
    }

    /// Handle mouse button release
    pub fn handle_mouse_release(&mut self) -> Result<()> {
        if self.dragging_border {
            log::info!("Stopped dragging border");
            self.dragging_border = false;
        }

        // Stop text selection
        if self.selecting {
            self.selecting = false;
            log::debug!("Stopped selection, active: {}", self.selection.active);
        }

        Ok(())
    }

    /// Handle mouse movement (for dragging borders and text selection)
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) -> Result<bool> {
        let mut needs_redraw = false;

        // Handle border dragging
        if self.dragging_border {
            let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
            let (cell_width, cell_height) = self.renderer.cell_dimensions();
            let x_u32 = x as u32;
            let y_u32 = y as u32;

            if self.pane_manager.update_border_at(x_u32, y_u32, window_rect, cell_width, cell_height)? {
                log::debug!("Updated border position to ({}, {})", x, y);
                needs_redraw = true;
            }
        }

        // Handle text selection dragging
        if self.selecting {
            let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
            let rects = self.pane_manager.layout().calculate_rects(window_rect);

            // Find which pane the mouse is over
            for (pane_id, rect) in rects {
                if x >= rect.x as f64 && x < (rect.x + rect.width) as f64 &&
                   y >= rect.y as f64 && y < (rect.y + rect.height) as f64 {
                    // Only update if it's the active pane
                    if pane_id == self.pane_manager.active_pane_id() {
                        let (cell_width, cell_height) = self.renderer.cell_dimensions();
                        let col = ((x - rect.x as f64) / cell_width as f64) as usize;
                        let row = ((y - rect.y as f64) / cell_height as f64) as usize;

                        self.selection.update_end(col, row);
                        needs_redraw = true;
                    }
                    break;
                }
            }
        }

        Ok(needs_redraw)
    }

    /// Change font size by delta
    fn change_font_size(&mut self, delta: f32) -> Result<()> {
        let current_size = self.renderer.font_size();
        let new_size = (current_size + delta).clamp(8.0, 32.0);

        if new_size != current_size {
            log::info!("Changing font size from {} to {}", current_size, new_size);
            self.renderer.set_font_size(new_size)?;

            // Recalculate all pane sizes with new cell dimensions
            let window_rect = Rect::new(0, 0, self.window_width, self.window_height);
            let (cell_width, cell_height) = self.renderer.cell_dimensions();
            self.pane_manager.resize_all_panes(window_rect, cell_width, cell_height)?;
        }

        Ok(())
    }

    /// Check if broadcast mode is enabled
    pub fn is_broadcast_enabled(&self) -> bool {
        self.pane_manager.is_broadcast_enabled()
    }

    /// Get the base window title
    pub fn base_title(&self) -> &str {
        &self.config.window.title
    }

    /// Handle copy operation
    fn handle_copy(&mut self) -> Result<()> {
        if !self.selection.active {
            log::debug!("No active selection to copy");
            return Ok(());
        }

        // Get text from active pane's grid
        if let Some(pane) = self.pane_manager.active_pane() {
            let text = self.selection.get_text(pane.terminal().grid());

            if !text.is_empty() {
                self.clipboard_manager.copy(&text)?;
                log::info!("Copied {} bytes to clipboard", text.len());
            } else {
                log::debug!("Selection is empty, nothing to copy");
            }
        }

        // Clear selection after copy
        self.selection.clear();

        Ok(())
    }

    /// Handle paste operation
    fn handle_paste(&mut self) -> Result<()> {
        match self.clipboard_manager.paste() {
            Ok(text) => {
                if !text.is_empty() {
                    // Write pasted text to active pane(s)
                    self.pane_manager.write_input(text.as_bytes())?;
                    log::info!("Pasted {} bytes from clipboard", text.len());
                } else {
                    log::debug!("Clipboard is empty, nothing to paste");
                }
            }
            Err(e) => {
                log::warn!("Failed to paste: {}", e);
                // Don't fail the operation, just log the error
            }
        }

        Ok(())
    }

    /// Toggle mark mode (keyboard-based text selection)
    fn toggle_mark_mode(&mut self) {
        if self.mark_mode {
            // Exiting mark mode
            self.mark_mode = false;
            self.mark_cursor = None;
            self.selection.clear();
            log::info!("Mark mode disabled");
        } else {
            // Entering mark mode
            self.mark_mode = true;

            // Initialize mark cursor at current terminal cursor position
            if let Some(pane) = self.pane_manager.active_pane() {
                let (col, row) = pane.terminal().cursor_position();
                self.mark_cursor = Some((col, row));
                self.selection.start_at(col, row);
                log::info!("Mark mode enabled at ({}, {})", col, row);
            }
        }
    }

    /// Handle arrow key navigation in mark mode
    fn handle_mark_mode_navigation(&mut self, key_code: winit::keyboard::KeyCode) -> bool {
        if !self.mark_mode {
            return false;
        }

        let Some((mut col, mut row)) = self.mark_cursor else {
            return false;
        };

        let Some(pane) = self.pane_manager.active_pane() else {
            return false;
        };

        let grid = pane.terminal().grid();
        let max_col = grid.cols().saturating_sub(1);
        let max_row = grid.rows().saturating_sub(1);

        // Move cursor based on arrow key
        match key_code {
            winit::keyboard::KeyCode::ArrowLeft => {
                col = col.saturating_sub(1);
            }
            winit::keyboard::KeyCode::ArrowRight => {
                col = (col + 1).min(max_col);
            }
            winit::keyboard::KeyCode::ArrowUp => {
                row = row.saturating_sub(1);
            }
            winit::keyboard::KeyCode::ArrowDown => {
                row = (row + 1).min(max_row);
            }
            winit::keyboard::KeyCode::Enter => {
                // Copy selection and exit mark mode
                let _ = self.handle_copy();
                self.mark_mode = false;
                self.mark_cursor = None;
                log::info!("Mark mode: copied selection and exited");
                return true;
            }
            winit::keyboard::KeyCode::Escape => {
                // Exit mark mode without copying
                self.mark_mode = false;
                self.mark_cursor = None;
                self.selection.clear();
                log::info!("Mark mode: exited without copying");
                return true;
            }
            _ => return false,
        }

        // Update mark cursor and selection
        self.mark_cursor = Some((col, row));
        self.selection.update_end(col, row);
        log::debug!("Mark mode cursor moved to ({}, {})", col, row);

        true
    }

    /// Check if mark mode is active
    pub fn is_mark_mode_active(&self) -> bool {
        self.mark_mode
    }

    /// Set IME (Input Method Editor) enabled/disabled (called by OS events)
    pub fn set_ime_enabled(&mut self, enabled: bool) {
        self.ime_enabled = enabled;
        log::info!("IME {}", if self.ime_enabled { "enabled" } else { "disabled" });
    }

    /// Check if IME is enabled
    pub fn is_ime_enabled(&self) -> bool {
        self.ime_enabled
    }

    /// Handle IME commit (when user confirms input)
    pub fn handle_ime_commit(&mut self, text: String) -> Result<()> {
        if !text.is_empty() {
            log::debug!("Writing IME commit to PTY: {:?} ({} bytes)", text, text.len());
            self.pane_manager.write_input(text.as_bytes())?;
            log::debug!("Successfully wrote IME commit to PTY");
        } else {
            log::debug!("Empty IME commit, ignoring");
        }
        Ok(())
    }

    /// Get cursor position in pixels for IME cursor area
    /// Returns (x, y) in physical pixels
    pub fn get_ime_cursor_position(&self) -> (f32, f32) {
        let (cell_width, cell_height) = self.renderer.cell_dimensions();
        let window_rect = Rect::new(0, 0, self.window_width, self.window_height);

        if let Some(pane) = self.pane_manager.active_pane() {
            if let Some(rect) = self.pane_manager.active_pane_rect(window_rect) {
                let (col, row) = pane.terminal().cursor_position();

                // Calculate pixel position
                let x = rect.x as f32 + (col as f32 * cell_width);
                let y = rect.y as f32 + (row as f32 * cell_height);

                (x, y)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        }
    }

    /// Get cell dimensions (width, height) in pixels
    pub fn cell_dimensions(&self) -> (f32, f32) {
        self.renderer.cell_dimensions()
    }

}
