use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub renderer: RendererConfig,

    #[serde(default)]
    pub terminal: TerminalConfig,

    #[serde(default)]
    pub window: WindowConfig,

    #[serde(default)]
    pub startup: StartupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    /// Backend selection: "auto", "gpu", or "cpu"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Target FPS
    #[serde(default = "default_target_fps")]
    pub target_fps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Number of columns
    #[serde(default = "default_cols")]
    pub cols: usize,

    /// Number of rows
    #[serde(default = "default_rows")]
    pub rows: usize,

    /// Font size in pixels
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    /// Font family
    #[serde(default = "default_font_family")]
    pub font_family: String,

    /// Scrollback buffer size
    #[serde(default = "default_scrollback")]
    pub scrollback: usize,

    /// Shell command to execute
    #[serde(default = "default_shell")]
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    #[serde(default = "default_title")]
    pub title: String,

    /// Initial window width
    #[serde(default = "default_width")]
    pub width: u32,

    /// Initial window height
    #[serde(default = "default_height")]
    pub height: u32,

    /// Maximize window on startup
    #[serde(default = "default_maximize")]
    pub maximize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Number of panes to create on startup (1, 2, or 4)
    #[serde(default = "default_panes")]
    pub panes: usize,

    /// Layout type: "single", "horizontal", "vertical", "grid"
    #[serde(default = "default_layout")]
    pub layout: String,

    /// Split ratio for horizontal splits (e.g., 0.7 for 7:3 ratio)
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f32,

    /// Split ratio for vertical splits (e.g., 0.5 for 5:5 ratio)
    #[serde(default = "default_vertical_ratio")]
    pub vertical_ratio: f32,
}

// Default functions
fn default_backend() -> String {
    "auto".to_string()
}

fn default_target_fps() -> u32 {
    60
}

fn default_cols() -> usize {
    80
}

fn default_rows() -> usize {
    24
}

fn default_font_size() -> f32 {
    14.0
}

fn default_font_family() -> String {
    "monospace".to_string()
}

fn default_scrollback() -> usize {
    10000
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

fn default_title() -> String {
    "terbulator".to_string()
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    600
}

fn default_maximize() -> bool {
    true
}

fn default_panes() -> usize {
    4
}

fn default_layout() -> String {
    "grid".to_string()
}

fn default_split_ratio() -> f32 {
    0.7
}

fn default_vertical_ratio() -> f32 {
    0.5
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            target_fps: default_target_fps(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            cols: default_cols(),
            rows: default_rows(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            scrollback: default_scrollback(),
            shell: default_shell(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            width: default_width(),
            height: default_height(),
            maximize: default_maximize(),
        }
    }
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            panes: default_panes(),
            layout: default_layout(),
            split_ratio: default_split_ratio(),
            vertical_ratio: default_vertical_ratio(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            renderer: RendererConfig::default(),
            terminal: TerminalConfig::default(),
            window: WindowConfig::default(),
            startup: StartupConfig::default(),
        }
    }
}
