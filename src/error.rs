use std::fmt::Display;

use spinoff::{Spinner, Color};

use crate::manager::DOTS;

pub enum Error {
    Context(String, Box<Error>),
    Custom(String),
    Reqwest(reqwest::Error),
    Json(Box<dyn std::error::Error + Send>),
    Io(std::io::Error),
}

impl Error {
    pub fn context(ctx: impl Display, error: impl Into<Error>) -> Self {
        Self::Context(ctx.to_string(), Box::new(error.into()))
    }

    pub fn custom(message: impl Display) -> Self {
        Self::Custom(message.to_string())
    }

    pub fn update_spinner(&self, spinner: &mut Spinner, msg: impl std::fmt::Display) {
        spinner.fail(format!("{msg}\n\x1b[31m  {self}\x1b[39m").as_str());
        *spinner = Spinner::new(DOTS, msg.to_string(), Color::Yellow);
    }
}

pub trait UpdateSpinner<T> {
    fn ok_with_spinner(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Option<T>;
}

impl<T> UpdateSpinner<T> for Result<T, Error> {
    fn ok_with_spinner(self, spinner: &mut Spinner, msg: impl std::fmt::Display) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                spinner.fail(format!("{msg}\n\x1b[31m  {err}\x1b[39m").as_str());
                *spinner = Spinner::new(DOTS, msg.to_string(), Color::Yellow);
                None
            }
        }
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
