use std::path::PathBuf;

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the application
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError { file: PathBuf, message: String },
    InvalidArgument(String),
    FrameworkNotDetected,
    SerializationError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO 错误: {}", e),
            Error::ParseError { file, message } => {
                write!(f, "解析错误 {}: {}", file.display(), message)
            }
            Error::InvalidArgument(msg) => write!(f, "无效参数: {}", msg),
            Error::FrameworkNotDetected => write!(f, "未检测到支持的 Web 框架"),
            Error::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(format!("JSON 序列化错误: {}", err))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::SerializationError(format!("YAML 序列化错误: {}", err))
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error::ParseError {
            file: PathBuf::from("<unknown>"),
            message: err.to_string(),
        }
    }
}
