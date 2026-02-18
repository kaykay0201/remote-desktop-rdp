use std::io;

pub type Result<T> = std::result::Result<T, RdpError>;

#[derive(Debug, thiserror::Error)]
pub enum RdpError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("authentication failed: {0}")]
    Authentication(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("disconnected")]
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_connection_error() {
        let err = RdpError::Connection("timeout".to_string());
        assert_eq!(err.to_string(), "connection failed: timeout");
    }

    #[test]
    fn display_auth_error() {
        let err = RdpError::Authentication("bad password".to_string());
        assert_eq!(err.to_string(), "authentication failed: bad password");
    }

    #[test]
    fn display_tls_error() {
        let err = RdpError::Tls("cert invalid".to_string());
        assert_eq!(err.to_string(), "TLS error: cert invalid");
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let rdp_err: RdpError = io_err.into();
        assert!(matches!(rdp_err, RdpError::Io(_)));
        assert!(rdp_err.to_string().contains("refused"));
    }
}
