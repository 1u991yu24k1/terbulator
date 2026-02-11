use crate::terminal::{PtyController, TerminalEmulator};
use crate::utils::Result;

pub type PaneId = usize;

/// 個別のペイン（独立した端末エミュレータとPTYを持つ）
pub struct Pane {
    id: PaneId,
    terminal: TerminalEmulator,
    pty: PtyController,
    is_active: bool,
    needs_redraw: bool, // Whether this pane needs to be redrawn
}

impl Pane {
    pub fn new(id: PaneId, cols: usize, rows: usize, scrollback: usize, shell: &str) -> Result<Self> {
        log::info!("Creating pane {} with size {}x{}, shell: {}", id, cols, rows, shell);
        let terminal = TerminalEmulator::new(cols, rows, scrollback);

        log::info!("Initializing PTY for pane {}", id);
        let pty = match PtyController::new(cols as u16, rows as u16, shell) {
            Ok(p) => {
                log::info!("PTY successfully created for pane {}", id);
                p
            }
            Err(e) => {
                log::error!("Failed to create PTY for pane {}: {}", id, e);
                return Err(e);
            }
        };

        log::info!("Pane {} created successfully: {}x{}", id, cols, rows);

        Ok(Self {
            id,
            terminal,
            pty,
            is_active: false,
            needs_redraw: true, // Initial draw needed
        })
    }

    pub fn id(&self) -> PaneId {
        self.id
    }

    pub fn terminal(&self) -> &TerminalEmulator {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut TerminalEmulator {
        &mut self.terminal
    }

    pub fn pty_mut(&mut self) -> &mut PtyController {
        &mut self.pty
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<()> {
        log::info!("Resizing pane {} from current size to {}x{}", self.id, cols, rows);
        self.terminal.resize(cols, rows);
        match self.pty.resize(cols as u16, rows as u16) {
            Ok(_) => {
                log::debug!("Successfully resized PTY for pane {}", self.id);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to resize PTY for pane {}: {}", self.id, e);
                Err(e)
            }
        }
    }

    /// Check if the pane's PTY process is still alive
    pub fn is_alive(&mut self) -> bool {
        let alive = self.pty.is_alive();
        if !alive {
            log::info!("Pane {} PTY process has exited", self.id);
        }
        alive
    }

    pub fn process_pty_output(&mut self) -> Result<bool> {
        let mut buf = [0u8; 4096];
        let mut has_output = false;
        let mut total_read = 0;
        const MAX_READ_PER_FRAME: usize = 64 * 1024; // 最大64KB/フレーム

        // Non-blocking read - try to read multiple times but limit total amount
        loop {
            match self.pty.read(&mut buf) {
                Ok(n) if n > 0 => {
                    self.terminal.process_bytes(&buf[..n]);
                    has_output = true;
                    total_read += n;

                    // Stop if we've read enough for this frame to remain responsive
                    if total_read >= MAX_READ_PER_FRAME {
                        log::trace!("Pane {} read limit reached ({}KB), deferring rest", self.id, total_read / 1024);
                        break;
                    }

                    // Continue reading if buffer was full
                    if n < buf.len() {
                        break;
                    }
                }
                Ok(_) => break,
                Err(e) => {
                    // Check if it's just "would block" error (no data available)
                    if let crate::utils::TerbulatorError::Io(io_err) = &e {
                        if io_err.kind() == std::io::ErrorKind::WouldBlock {
                            break; // No more data available
                        }
                        log::error!("Pane {} PTY read error: {}", self.id, io_err);
                        return Err(e);
                    } else {
                        log::error!("Pane {} error: {}", self.id, e);
                        return Err(e);
                    }
                }
            }
        }

        if total_read > 0 {
            log::trace!("Pane {} read {} bytes from PTY", self.id, total_read);
        }

        // Mark for redraw if there was output
        if has_output {
            self.needs_redraw = true;
        }

        Ok(has_output)
    }

    pub fn write_input(&self, data: &[u8]) -> Result<()> {
        self.pty.write(data)?;
        Ok(())
    }

    /// Check if pane needs redraw
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Mark pane as needing redraw
    pub fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }

    /// Clear the needs_redraw flag (after rendering)
    pub fn clear_redraw_flag(&mut self) {
        self.needs_redraw = false;
    }
}
