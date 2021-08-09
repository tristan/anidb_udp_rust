use std::num::ParseIntError;
use std::str::Utf8Error;
use tokio::sync::{
    oneshot::error::RecvError,
    mpsc::error::SendError,
};

#[derive(Debug, thiserror::Error)]
pub enum AniDbError {
    // generic errors
    #[error("ILLEGAL INPUT OR ACCESS DENIED")]
    IllegalOutputOrAccessDenied,
    #[error("BANNED: {0}")]
    Banned(String),
    #[error("UNKNOWN COMMAND")]
    UnknownCommand,
    #[error("INTERNAL SERVER ERROR")]
    InternalServerError,
    #[error("ANIDB OUT OF SERVICE - TRY AGAIN LATER")]
    OutOfService,
    #[error("SERVER BUSY - TRY AGAIN LATER")]
    ServerBusy,
    #[error("TIMEOUT - DELAY AND RESUBMIT")]
    Timeout,
    #[error("LOGIN FIRST")]
    LoginFirst,
    #[error("ACCESS DENIED")]
    AccessDenied,
    #[error("INVALID SESSION")]
    InvalidSession,
    #[error("Server returned unknown error code: {0}")]
    UnknownError(String),
    // auth errors
    #[error("LOGIN FAILED")]
    LoginFailed,
    #[error("CLIENT VERSION OUTDATED")]
    ClientVersionOutdated,
    #[error("CLIENT BANNED: {0}")]
    ClientBanned(String),
    // other errors
    #[error("Cache Error: {0}")]
    CacheError(String),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    SerializationError(#[from] serde_urlencoded::ser::Error),
    #[error("Deserialization Error: {0}")]
    DecodeError(String),
    #[error("Unexpected Error: {0}")]
    UnexpectedError(String),
}

impl From<Utf8Error> for AniDbError {
    fn from(e: Utf8Error) -> Self {
        AniDbError::DecodeError(format!("{}", e))
    }
}

impl From<ParseIntError> for AniDbError {
    fn from(e: ParseIntError) -> Self {
        AniDbError::DecodeError(format!("{}", e))
    }
}

impl From<RecvError> for AniDbError {
    fn from(e: RecvError) -> Self {
        AniDbError::UnexpectedError(format!("{}", e))
    }
}

impl From<SendError<String>> for AniDbError {
    fn from(e: SendError<String>) -> Self {
        AniDbError::UnexpectedError(format!("{}", e))
    }
}

macro_rules! impl_into_anidberror_internal {
    ($name:ty, $typ:ty) => {
        impl From<$typ> for $name {
            fn from(e: $typ) -> $name {
                AniDbError::from(e).into()
            }
        }
    };
}

macro_rules! impl_into_anidberror {
    ($name:ty) => {
        impl_into_anidberror_internal!($name, std::str::Utf8Error);
        impl_into_anidberror_internal!($name, std::io::Error);
        impl_into_anidberror_internal!($name, std::num::ParseIntError);
        impl_into_anidberror_internal!($name, serde_urlencoded::ser::Error);
        impl_into_anidberror_internal!($name, tokio::sync::oneshot::error::RecvError);
        impl_into_anidberror_internal!($name, tokio::sync::mpsc::error::SendError<String>);
    }
}

impl From<(&str, &str)> for AniDbError {
    fn from((code, buf): (&str, &str)) -> Self {
        match code {
            "500" => AniDbError::LoginFailed,
            "501" => AniDbError::LoginFirst,
            "502" => AniDbError::AccessDenied,
            "503" => AniDbError::ClientVersionOutdated,
            "504" => AniDbError::ClientBanned(buf.to_string()),
            "505" => AniDbError::IllegalOutputOrAccessDenied,
            "506" => AniDbError::InvalidSession,
            "555" => AniDbError::Banned(buf.to_string()),
            "598" => AniDbError::UnknownCommand,
            "600" => AniDbError::InternalServerError,
            "601" => AniDbError::OutOfService,
            "602" => AniDbError::ServerBusy,
            "604" => AniDbError::Timeout,
            code @ _ => AniDbError::UnknownError(format!("{0} {1}", code, buf)),
        }
    }
}
