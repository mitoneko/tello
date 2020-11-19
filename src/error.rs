#[derive(Clone)]
pub enum TelloError {
    SocketError(String),
    TelloCmdFail(String),
    TelloResponsIllegal(String),
    TelloTimeout(String),
}

impl From<std::io::Error> for TelloError {
    fn from(e: std::io::Error) -> Self {
        Self::SocketError(e.to_string())
    }
}

impl std::fmt::Display for TelloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TelloError::*;
        match self {
            SocketError(e) => write!(f, "SocketError: {}", e),
            TelloCmdFail(s) => write!(f, "Tello Error: {}", s),
            TelloResponsIllegal(s) => write!(f, "Illegal Respons from tello.[{}]", s),
            TelloTimeout(s) => write!(f, "Timeout: {}", s),
        }
    }
}

impl std::fmt::Debug for TelloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for TelloError {}
