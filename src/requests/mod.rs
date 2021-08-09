pub mod types;
pub mod auth;
pub mod anime;
pub mod file;

use std::{
    fmt,
    error,
    io
};
use serde::Serialize;
use tokio::sync::{
    oneshot::error::RecvError,
    mpsc::error::SendError,
};
use crate::AniDbError;

pub trait AniDbRequest: Serialize {
    type Error: fmt::Debug + error::Error + From<AniDbError> + From<serde_urlencoded::ser::Error> + From<io::Error> + From<RecvError> + From<SendError<String>>;
    type Response;
    fn name() -> &'static str;
    fn requires_login() -> bool {
        true
    }
    fn encode(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(self)
    }
    fn decode_response(
        &self,
        code: &str,
        reply: &str,
        data: &str
    ) -> Result<Self::Response, Self::Error>;
}
