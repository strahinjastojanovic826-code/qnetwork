use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    Io(String),
    PackageTooShort,
    IncorrectMagicHeader,
    OverflowPayloadCalculation,
    TotalLengthExceedsLimits,
    PackageSmallerThanHeader,
    ChecksumMismatch,
    InsufficientQuats,
    InvalidRleStream,
    ConnectionClosed,
    SetTimeoutFailed,
    BindFailed,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::Io(err) => write!(f, "IO error: {}", err),
            NetworkError::PackageTooShort => write!(f, "The package is too short."),
            NetworkError::IncorrectMagicHeader => write!(f, "Incorrect Magic header."),
            NetworkError::OverflowPayloadCalculation => write!(f, "Overflow payload calculation."),
            NetworkError::TotalLengthExceedsLimits => write!(f, "The total length exceeds the permitted limits."),
            NetworkError::PackageSmallerThanHeader => write!(f, "The package is smaller than the header indicates."),
            NetworkError::ChecksumMismatch => write!(f, "Checksum error: The data is corrupted."),
            NetworkError::InsufficientQuats => write!(f, "Insufficient quats for target data type."),
            NetworkError::InvalidRleStream => write!(f, "Invalid RLE stream length."),
            NetworkError::ConnectionClosed => write!(f, "Connection closed."),
            NetworkError::SetTimeoutFailed => write!(f, "Failed to set socket timeout."),
            NetworkError::BindFailed => write!(f, "Failed to bind socket."),
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<std::io::Error> for NetworkError {
    fn from(err: std::io::Error) -> Self {
        NetworkError::Io(err.to_string())
    }
}