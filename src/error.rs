use std::fmt::Display;

pub enum Error {
    Context(String, Box<Error>),
    Custom(String),
    Reqwest(reqwest::Error),
    Json(Box<dyn std::error::Error + Send>),
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl Error {
    pub fn context(ctx: impl Display, error: impl Into<Error>) -> Self {
        Self::Context(ctx.to_string(), Box::new(error.into()))
    }

    pub fn custom(message: impl Display) -> Self {
        Self::Custom(message.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(req) => write!(f, "{req}"),
            Self::Json(json) => write!(f, "{json}"),
            Self::Io(io) => write!(f, "{io}"),
            Self::Context(context, err) => write!(f, "ctx: {context}\n{err}"),
            Self::Custom(message) => write!(f, "{message}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(Box::new(value))
    }
}

impl<E: std::error::Error + Send + 'static> From<serde_path_to_error::Error<E>> for Error {
    fn from(value: serde_path_to_error::Error<E>) -> Self {
        Self::Json(Box::new(value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
