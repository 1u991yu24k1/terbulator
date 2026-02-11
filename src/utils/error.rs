use thiserror::Error;

pub type Result<T> = std::result::Result<T, TerbulatorError>;

#[derive(Error, Debug)]
pub enum TerbulatorError {
    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("PTY error: {0}")]
    Pty(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Window error: {0}")]
    Window(String),

    #[error("Backend initialization failed: {0}")]
    BackendInit(String),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

// Convenience constructors
impl TerbulatorError {
    pub fn rendering(msg: impl Into<String>) -> Self {
        Self::Rendering(msg.into())
    }

    pub fn terminal(msg: impl Into<String>) -> Self {
        Self::Terminal(msg.into())
    }

    pub fn pty(msg: impl Into<String>) -> Self {
        Self::Pty(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn window(msg: impl Into<String>) -> Self {
        Self::Window(msg.into())
    }

    pub fn backend_init(msg: impl Into<String>) -> Self {
        Self::BackendInit(msg.into())
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }
}
