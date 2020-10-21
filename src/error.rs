pub enum TelloError {
    SocketError(std::io::Error),
    TelloCmdFail(String),
    TelloResponsIllegal(String),
}

impl From<std::io::Error> for TelloError {
    fn from(e: std::io::Error) -> Self {
        Self::SocketError(e)
    }
}

impl std::fmt::Display for TelloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TelloError::*;
        match self {
            SocketError(e) => write!(f, "SocketError: {}", e),
            TelloCmdFail(s) => write!(f, "Tello Error: {}", s),
            TelloResponsIllegal(s) => write!(f, "Illegal Respons from tello.[{}]", s),
        }
    }
}

impl std::fmt::Debug for TelloError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
