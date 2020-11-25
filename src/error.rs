use actix_web_actors::ws::ProtocolError;


pub type Result<T> = std::result::Result<T, TSError>;

/// TSError enumerates all possible errors returned by this library
#[derive(Debug)]
pub enum TSError {
    ConfigError(::config::ConfigError),
    Error(String),

    /// Represents all other cases of IO Error
    IOError(std::io::Error),

    /// Represents all other cases of websocket ProtocolError
    ProtocolError(ProtocolError),
}

impl std::error::Error for TSError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            TSError::ConfigError(ref err) => Some(err),
            TSError::Error(_) => None,
            TSError::IOError(ref err) => Some(err),
            TSError::ProtocolError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for TSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TSError::ConfigError(ref err) => err.fmt(f),
            TSError::Error(ref msg) => write!(f, "{}", msg),
            TSError::IOError(ref err) => err.fmt(f),
            TSError::ProtocolError(ref err) => err.fmt(f),
        }
    }
}

impl From<::config::ConfigError> for TSError {
    fn from(err: ::config::ConfigError) -> TSError {
        TSError::ConfigError(err)
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

pub trait UnwrapOrExit<T>
where
    Self: Sized,
{
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T;

    fn unwrap_or_exit(self, message: &str) -> T {
        let err = clap::Error::with_description(message, clap::ErrorKind::InvalidValue);
        self.unwrap_or_else(|| err.exit())
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.unwrap_or_else(f)
    }
}

impl<T> UnwrapOrExit<T> for Result<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.unwrap_or_else(|_| f())
    }

    fn unwrap_or_exit(self, message: &str) -> T {
        self.unwrap_or_else(|e| {
            let err = clap::Error::with_description(
                &format!("{}: {}", message, e),
                clap::ErrorKind::InvalidValue,
            );
            err.exit()
        })
    }
}
