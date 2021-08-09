#[cfg(feature = "ed2k")]
pub mod ed2k;
#[cfg(feature = "crc")]
pub mod crc;

mod client;
mod cache;
#[macro_use]
mod mask;
#[macro_use]
mod errors;
mod requests;

pub use async_trait::async_trait;

pub use crate::client::AniDbClient;
pub use crate::cache::AniDbCache;
pub use crate::errors::AniDbError;
pub use crate::requests::{
    auth::{
        AuthRequest,
        AuthResponse,
    },
    anime::{
        AnimeRequest,
        AnimeRequestError,
        AnimeRequestFields,
        AnimeResponse,
    },
    file::{
        FileRequest,
        FileResponse,
        FileMask,
        AnimeMask,
        FileMaskResponse,
        AnimeMaskResponse,
        FileRequestError,
    },
    types::EpNo,
};

const ANIDB_ADDR: (&str, u16) = ("api.anidb.net", 9000);
const ANIDB_API_VERSION: i32 = 3;
const CLIENT_NAME: &str = "anidbudprust";
const CLIENT_VERSION: i32 = 1;
