use crate::utils::{Result, TerbulatorError};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PtyController {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send>,
    rx: Receiver<Vec<u8>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PtyController {
    pub fn new(cols: u16, rows: u16, shell: &str) -> Result<Self> {
        log::info!("PtyController::new() called with cols={}, rows={}, shell={}", cols, rows, shell);
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        log::debug!("Opening PTY with size {}x{}", cols, rows);
        let pair = pty_system
            .openpty(pty_size)
            .map_err(|e| TerbulatorError::pty(format!("Failed to open PTY: {}", e)))?;

        log::debug!("Spawning shell: {}", shell);
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| TerbulatorError::pty(format!("Failed to spawn shell '{}': {}", shell, e)))?;

        log::debug!("Shell spawned successfully, PID: {:?}", child.process_id());

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| TerbulatorError::pty(format!("Failed to clone reader: {}", e)))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| TerbulatorError::pty(format!("Failed to take writer: {}", e)))?;

        // Create a channel for sending data from PTY reader thread
        let (tx, rx) = channel();

        // Spawn a thread to read from PTY
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let data = buf[..n].to_vec();
                        if tx.send(data).is_err() {
                            log::error!("Failed to send PTY data, channel closed");
                            break;
                        }
                    }
                    Ok(_) => {
                        // EOF
                        log::info!("PTY reader reached EOF");
                        break;
                    }
                    Err(e) => {
                        log::error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
            log::info!("PTY reader thread exiting");
        });

        log::info!("PTY initialized: {}x{} shell={}", cols, rows, shell);

        Ok(Self {
            master: pair.master,
            child,
            rx,
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    /// Check if the child process is still alive
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_exit_status)) => {
                log::info!("Child process has exited");
                false
            }
            Ok(None) => {
                // Still running
                true
            }
            Err(e) => {
                log::error!("Failed to check child process status: {}", e);
                false
            }
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        self.master
            .resize(pty_size)
            .map_err(|e| TerbulatorError::pty(format!("Failed to resize PTY: {}", e)))?;

        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.rx.try_recv() {
            Ok(data) => {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            }
            Err(TryRecvError::Empty) => {
                // No data available - return WouldBlock error
                Err(TerbulatorError::Io(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "No data available",
                )))
            }
            Err(TryRecvError::Disconnected) => {
                // Channel closed - EOF
                Ok(0)
            }
        }
    }

    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|e| TerbulatorError::pty(format!("Failed to lock writer: {}", e)))?;

        writer
            .write(data)
            .map_err(|e| TerbulatorError::pty(format!("Failed to write to PTY: {}", e)))?;

        writer
            .flush()
            .map_err(|e| TerbulatorError::pty(format!("Failed to flush PTY: {}", e)))?;

        Ok(data.len())
    }

    pub fn get_writer(&self) -> Arc<Mutex<Box<dyn Write + Send>>> {
        Arc::clone(&self.writer)
    }
}
