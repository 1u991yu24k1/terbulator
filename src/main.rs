mod app;
mod clipboard;
mod config;
mod input;
mod pane;
mod renderer;
mod terminal;
mod utils;

use app::App;
use config::init_config;
use clap::Parser;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Terbulator - 超軽量なGUI端末エミュレータ
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path (default: ~/.config/terbulator/config.yaml)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

struct TerbulatorApp {
    window: Option<Window>,
    app: Option<App>,
    modifiers: winit::keyboard::ModifiersState,
    config_path: Option<PathBuf>,
    cursor_position: (f64, f64),
    last_cursor_blink: Instant,
    cursor_blink_interval: Duration,
}

impl ApplicationHandler for TerbulatorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Load config
            let config = match init_config(self.config_path.clone()) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::error!("Failed to load config: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            // Create window
            let mut window_attrs = winit::window::WindowAttributes::default()
                .with_title(&config.window.title);

            // Set window size or maximize
            if config.window.maximize {
                window_attrs = window_attrs.with_maximized(true);
                log::info!("Creating maximized window");
            } else {
                window_attrs = window_attrs.with_inner_size(winit::dpi::PhysicalSize::new(
                    config.window.width,
                    config.window.height,
                ));
                log::info!("Creating window with size {}x{}", config.window.width, config.window.height);
            }

            let window = match event_loop.create_window(window_attrs) {
                Ok(w) => w,
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            // Create app
            let app = match App::new(config, &window) {
                Ok(a) => a,
                Err(e) => {
                    log::error!("Failed to create app: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            log::info!("Terbulator initialized with {:?} backend", app.backend_type());
            let (cols, rows) = app.grid_info();
            log::info!("Grid size: {}x{}", cols, rows);

            // Enable IME for Japanese input
            window.set_ime_allowed(true);
            // Set IME cursor area (position where IME popup appears)
            // Start at top-left corner, will be updated based on cursor position
            window.set_ime_cursor_area(winit::dpi::PhysicalPosition::new(0, 0), winit::dpi::PhysicalSize::new(1, 1));
            log::info!("IME support enabled");

            self.window = Some(window);
            self.app = Some(app);

            // Request initial redraw
            if let Some(window) = &self.window {
                log::info!("Requesting initial redraw");
                window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check if cursor should blink
        let now = Instant::now();
        if now.duration_since(self.last_cursor_blink) >= self.cursor_blink_interval {
            self.last_cursor_blink = now;
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }

        // Calculate time until next cursor blink
        let elapsed = now.duration_since(self.last_cursor_blink);
        let next_blink = self.cursor_blink_interval.saturating_sub(elapsed);

        // Wait until next cursor blink time or until an event occurs
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + next_blink));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app) = &mut self.app else {
            return;
        };

        let Some(window) = &self.window else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Err(e) = app.resize(size.width, size.height) {
                    log::error!("Failed to resize: {}", e);
                }
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                // Process PTY output
                let (has_output, should_exit) = match app.process_pty_output() {
                    Ok((has_output, should_exit)) => (has_output, should_exit),
                    Err(e) => {
                        log::error!("Failed to process PTY output: {}", e);
                        (false, false)
                    }
                };

                // Exit if all panes have closed
                if should_exit {
                    log::info!("All panes closed, exiting application");
                    event_loop.exit();
                    return;
                }

                // Render
                if let Err(e) = app.render() {
                    log::error!("Failed to render: {}", e);
                }

                // Update IME cursor area to match terminal cursor position
                let (cursor_x, cursor_y) = app.get_ime_cursor_position();
                let (cell_width, cell_height) = app.cell_dimensions();
                window.set_ime_cursor_area(
                    winit::dpi::PhysicalPosition::new(cursor_x as i32, cursor_y as i32),
                    winit::dpi::PhysicalSize::new(cell_width as u32, cell_height as u32)
                );

                // Request another redraw if there was PTY output
                // (data might still be coming)
                if has_output {
                    window.request_redraw();
                }
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
                app.update_modifiers(self.modifiers);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Err(e) = app.handle_keyboard_input(&event.physical_key, self.modifiers) {
                        log::error!("Failed to handle keyboard input: {}", e);
                    }

                    update_window_title(app, window);
                    window.request_redraw();
                }
            }

            WindowEvent::Ime(ime) => {
                match ime {
                    winit::event::Ime::Enabled => {
                        log::info!("========================================");
                        log::info!("IME ENABLED by OS (e.g., Ctrl-Space pressed)");
                        log::info!("========================================");
                        app.set_ime_enabled(true);
                        update_window_title(app, window);
                        window.request_redraw();
                    }
                    winit::event::Ime::Disabled => {
                        log::info!("========================================");
                        log::info!("IME DISABLED by OS");
                        log::info!("========================================");
                        app.set_ime_enabled(false);
                        update_window_title(app, window);
                        window.request_redraw();
                    }
                    winit::event::Ime::Commit(text) => {
                        log::info!("========================================");
                        log::info!("IME COMMIT: {:?} ({} bytes)", text, text.len());
                        log::info!("========================================");
                        if let Err(e) = app.handle_ime_commit(text) {
                            log::error!("Failed to handle IME commit: {}", e);
                        }
                        window.request_redraw();
                    }
                    winit::event::Ime::Preedit(text, cursor) => {
                        if !text.is_empty() {
                            log::info!("IME preedit: {:?}, cursor: {:?}", text, cursor);
                        }
                        // Preedit (conversion candidates) are displayed by the OS IME
                        // We don't need to render them ourselves
                    }
                }
            }

            WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. } => {
                let (x, y) = self.cursor_position;
                match state {
                    ElementState::Pressed => {
                        // Handle left mouse button press
                        if let Err(e) = app.handle_mouse_press(x, y) {
                            log::error!("Failed to handle mouse press: {}", e);
                        }
                        window.request_redraw();
                    }
                    ElementState::Released => {
                        // Handle left mouse button release
                        if let Err(e) = app.handle_mouse_release() {
                            log::error!("Failed to handle mouse release: {}", e);
                        }
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                // Store cursor position
                self.cursor_position = (position.x, position.y);

                // Handle mouse move for dragging borders
                match app.handle_mouse_move(position.x, position.y) {
                    Ok(true) => {
                        // Border was updated, request redraw
                        window.request_redraw();
                    }
                    Ok(false) => {
                        // No update needed
                    }
                    Err(e) => {
                        log::error!("Failed to handle mouse move: {}", e);
                    }
                }
            }

            _ => {}
        }
    }
}

/// Update window title based on app state (broadcast, mark mode, IME)
fn update_window_title(app: &App, window: &Window) {
    let base_title = app.base_title();
    let mut title_parts = vec![];

    if app.is_broadcast_enabled() {
        title_parts.push("Broadcasting");
    }

    if app.is_mark_mode_active() {
        title_parts.push("MARK");
    }

    if app.is_ime_enabled() {
        title_parts.push("あ");
        log::debug!("IME is enabled, adding [あ] to title");
    } else {
        log::debug!("IME is disabled, no [あ] in title");
    }

    let new_title = if title_parts.is_empty() {
        base_title.to_string()
    } else {
        format!("{} - [{}]", base_title, title_parts.join("] ["))
    };

    log::debug!("Setting window title to: {:?}", new_title);
    window.set_title(&new_title);
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger with debug level temporarily
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Starting terbulator");
    if let Some(ref config_path) = args.config {
        log::info!("Using config file: {}", config_path.display());
    }

    // Create event loop
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {}", e);
            std::process::exit(1);
        }
    };

    let mut app = TerbulatorApp {
        window: None,
        app: None,
        modifiers: winit::keyboard::ModifiersState::empty(),
        config_path: args.config,
        cursor_position: (0.0, 0.0),
        last_cursor_blink: Instant::now(),
        cursor_blink_interval: Duration::from_millis(500),
    };

    // Run event loop
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Event loop error: {}", e);
        std::process::exit(1);
    }
}
