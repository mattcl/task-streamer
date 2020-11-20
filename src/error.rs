use actix_web_actors::ws::ProtocolError;


pub type Result<T> = std::result::Result<T, TSError>;

/// TSError enumerates all possible errors returned by this library
#[derive(Debug)]
pub enum TSError {
    MyError(),
    Error(String),

    /// Represents all other cases of IO Error
    IOError(std::io::Error),

    /// Represents all other cases of websocket ProtocolError
    ProtocolError(ProtocolError),
}

impl std::error::Error for TSError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            TSError::MyError() => None,
            TSError::Error(_) => None,
            TSError::IOError(ref err) => Some(err),
            TSError::ProtocolError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for TSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TSError::MyError() => write!(f, "some error message"),
            TSError::Error(ref msg) => write!(f, "{}", msg),
            TSError::IOError(ref err) => err.fmt(f),
            TSError::ProtocolError(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for TSError {
    fn from(err: std::io::Error) -> TSError {
        TSError::IOError(err)
    }
}

impl From<ProtocolError> for TSError {
    fn from(err: ProtocolError) -> TSError {
        TSError::ProtocolError(err)
    }
}
