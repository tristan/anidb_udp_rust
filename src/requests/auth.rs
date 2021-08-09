use serde::{Serialize, Serializer};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;
use super::AniDbRequest;
use crate::{
    errors::AniDbError,
    ANIDB_API_VERSION,
    CLIENT_VERSION,
    CLIENT_NAME,
};

fn opt_bool_to_int<S>(x: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    x.map(|v| match v {
        true => 1,
        false => 0
    }).serialize(s)
}

#[derive(Clone, Serialize)]
// One of https://docs.oracle.com/javase/1.5.0/docs/guide/intl/encoding.doc.html
pub enum Encoding {
    #[serde(rename="UTF8")]
    Utf8
}

impl Default for Encoding {
    fn default() -> Encoding {
        Encoding::Utf8
    }
}

#[skip_serializing_none]
#[derive(Clone, Serialize, TypedBuilder)]
pub struct AuthRequest {
    user: String,
    pass: String,
    #[builder(default=ANIDB_API_VERSION)]
    protover: i32,
    #[builder(default_code="CLIENT_NAME.to_string()")]
    client: String,
    #[builder(default=CLIENT_VERSION)]
    clientver: i32,
    #[builder(default, setter(strip_option))]
    #[serde(serialize_with = "opt_bool_to_int")]
    nat: Option<bool>,
    #[builder(default, setter(strip_option))]
    #[serde(serialize_with = "opt_bool_to_int")]
    comp: Option<bool>,
    #[builder(default)]
    enc: Encoding,
    #[builder(default, setter(strip_option))]
    mtu: Option<i32>,
    #[serde(serialize_with = "opt_bool_to_int")]
    #[builder(default, setter(strip_option))]
    imgserver: Option<bool>,
}

#[derive(Debug)]
pub struct AuthResponse {
    pub session_id: String,
    pub new_version_available: bool,
    pub nat: Option<(String, u16)>,
}

impl AniDbRequest for AuthRequest {
    type Response = AuthResponse;
    type Error = AniDbError;

    fn name() -> &'static str {
        "AUTH"
    }

    fn requires_login() -> bool {
        false
    }

    fn decode_response(
        &self,
        code: &str,
        reply: &str,
        _data: &str
    ) -> Result<Self::Response, Self::Error> {
        match &code[..] {
            code @ ("200" | "201") => {
                let mut reply_iter = reply.split(" ");
                let session_str = reply_iter.next()
                    .ok_or_else(|| AniDbError::DecodeError(String::from("Invalid AUTH Reply")))?
                    .to_string();
                let ip_addr = if let Some(true) = self.nat {
                    let ip_str = reply_iter.next()
                        .ok_or_else(|| AniDbError::DecodeError(String::from("Invalid AUTH Reply")))?;
                    if ip_str == "LOGIN" {
                        return Err(AniDbError::DecodeError(String::from("Expected IP Address in AUTH Reply")));
                    }
                    let mut ip_iter = ip_str.split(":");
                    let ip_addr = ip_iter.next()
                        .ok_or_else(|| AniDbError::DecodeError(String::from("Invalid AUTH Reply")))?
                        .to_string();
                    let port = ip_iter.next()
                        .ok_or_else(|| AniDbError::DecodeError(String::from("Invalid AUTH Reply")))?
                        .parse()?;
                    Some((ip_addr, port))
                } else {
                    None
                };
                match code {
                    "200" => Ok(
                        AuthResponse {
                            session_id: session_str,
                            new_version_available: false,
                            nat: ip_addr
                        }
                    ),
                    "201" => Ok(
                        AuthResponse {
                            session_id: session_str,
                            new_version_available: true,
                            nat: ip_addr
                        }
                    ),
                    code @ _ => {
                        Err(AniDbError::from((code, reply)).into())
                    }
                }
            },
            code @ _ => Err(AniDbError::from((code, reply)).into())
        }
    }
}
