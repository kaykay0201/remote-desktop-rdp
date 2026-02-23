use std::io;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("capture error: {0}")]
    Capture(String),

    #[error("encoding error: {0}")]
    Encoding(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("input error: {0}")]
    Input(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("tailscale error: {0}")]
    Tailscale(String),

    #[error("disconnected")]
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_connection_error() {
        let err = AppError::Connection("timeout".to_string());
        assert_eq!(err.to_string(), "connection failed: timeout");
    }

    #[test]
    fn display_capture_error() {
        let err = AppError::Capture("no display".to_string());
        assert_eq!(err.to_string(), "capture error: no display");
    }

    #[test]
    fn display_network_error() {
        let err = AppError::Network("port in use".to_string());
        assert_eq!(err.to_string(), "network error: port in use");
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
        assert!(app_err.to_string().contains("refused"));
    }
}
