use std::num::ParseIntError;
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use super::AniDbRequest;
use crate::errors::AniDbError;
use super::types::EpNo;

generate_mask!(
    FileMask,
    FileMaskResponse,
    FileRequestError,
    6, i32, aid,
    5, i32, eid,
    4, i32, gid,
    3, i32, mylist_id,
    2, String, other_episodes,
    1, i16, is_deprecated,
    0, i16, state
        &
    7, i64, size,
    6, String, ed2k,
    5, String, md5,
    4, String, sha1,
    3, String, crc32,
    1, i32, video_colour_depth
        &
    7, String, quality,
    6, String, source,
    5, String, audio_codec_list,
    4, i32, audio_bitrate_list,
    3, String, video_codec,
    2, i32, video_bitrate,
    1, String, video_resolution,
    0, String, file_type,
    7, String, dub_language,
    6, String, sub_language,
    5, i32, length_in_seconds,
    4, String, description,
    3, i32, aired_date,
    0, String, anidb_file_name
        &
    7, i32, mylist_state,
    6, i32, mylist_filestate,
    5, i32, mylist_viewed,
    4, i32, mylist_viewdate,
    3, String, mylist_storage,
    2, String, mylist_source,
    1, String, mylist_other
        ;
);

generate_mask!(
    AnimeMask,
    AnimeMaskResponse,
    FileRequestError,
    7, i32, anime_total_episodes,
    6, i32, highest_episode_number,
    5, String, year,
    4, String, ty,
    3, String, related_aid_list,
    2, String, related_aid_type,
    1, String, category_list
        &
    7, String, romaji_name,
    6, String, kanji_name,
    5, String, english_name,
    4, String, other_name,
    3, String, short_name_list,
    2, String, synonym_list
        &
    7, EpNo, epno,
    6, String, ep_name,
    5, String, ep_romaji_name,
    4, String, ep_kanji_name,
    3, i32, episode_rating,
    2, i32, episode_vote_count
        &
    7, String, group_name,
    6, String, group_short_name,
    0, i32, date_aid_record_updated
        ;
);

pub enum FileRequest {
    Fid(i32, Option<FileMask>, Option<AnimeMask>),
    SizeEd2k(usize, String, Option<FileMask>, Option<AnimeMask>)
}

impl Serialize for FileRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let (base_len, fmask, amask) = match &self {
            FileRequest::Fid(_, fmask, amask) => {
                (1, fmask, amask)
            }
            FileRequest::SizeEd2k(_, _, fmask, amask) => {
                (2, fmask, amask)
            }
        };
        let len = base_len
            + if fmask.is_some() || amask.is_some() { 2 } else { 0 }
            ;

        let mut map = serializer.serialize_map(Some(len))?;
        let (fmask, amask) = match self {
            FileRequest::Fid(fid, fmask, amask) => {
                map.serialize_entry("fid", fid)?;
                (fmask, amask)
            },
            FileRequest::SizeEd2k(size, ed2k, fmask, amask) => {
                map.serialize_entry("size", size)?;
                map.serialize_entry("ed2k", ed2k)?;
                (fmask, amask)
            }
        };
        if let Some(fmask) = fmask {
            map.serialize_entry("fmask", fmask)?;
        } else if amask.is_some() {
            map.serialize_entry("fmask", &FileMask::none())?;
        }
        if let Some(amask) = amask {
            map.serialize_entry("amask", amask)?;
        } else if fmask.is_some() {
            map.serialize_entry("amask", &AnimeMask::none())?;
        }
        map.end()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileRequestError {
    #[error("NO SUCH FILE")]
    NoSuchFile,
    #[error("{0}")]
    AniDbError(#[from] AniDbError)
}
impl_into_anidberror!(FileRequestError);

pub enum FileResponse {
    File(i32, Option<FileMaskResponse>, Option<AnimeMaskResponse>),
    MultipleFiles(Vec<i32>),
}

impl AniDbRequest for FileRequest {
    type Response = FileResponse;
    type Error = FileRequestError;
    fn name() -> &'static str {
        "FILE"
    }
    fn decode_response(
        &self,
        code: &str,
        reply: &str,
        data: &str
    ) -> Result<Self::Response, Self::Error> {
        match &code[..] {
            "220" => {
                match &self {
                    &FileRequest::Fid(_, fmask, amask) | &FileRequest::SizeEd2k(_, _, fmask, amask) => {
                        let mut field_iter = data.trim().split("|");
                        let fid = next_or_decode_error!(field_iter)?.parse()?;
                        let (fresp, aresp) = match (fmask, amask) {
                            (Some(fmask), Some(amask)) => {
                                (
                                    Some(fmask.decode_response(&mut field_iter)?),
                                    Some(amask.decode_response(&mut field_iter)?)
                                )
                            },
                            (Some(fmask), None) => {
                                (
                                    Some(fmask.decode_response(&mut field_iter)?),
                                    None
                                )
                            },
                            (None, Some(amask)) => {
                                (
                                    None,
                                    Some(amask.decode_response(&mut field_iter)?)
                                )

                            },
                            (None, None) => {
                                let mut resp = FileMaskResponse::default();
                                decode_field!(field_iter, resp, aid, i32);
                                decode_field!(field_iter, resp, gid, i32);
                                decode_field!(field_iter, resp, state, i16);
                                decode_field!(field_iter, resp, size, i64);
                                decode_field!(field_iter, resp, ed2k, String);
                                decode_field!(field_iter, resp, anidb_file_name, String);
                                (
                                    Some(resp),
                                    None
                                )
                            }
                        };
                        Ok(FileResponse::File(fid, fresp, aresp))
                    }
                }
            },
            "322" => {
                Ok(FileResponse::MultipleFiles(
                    data.split("|")
                        .map(|v| v.parse())
                        .collect::<Result<_, ParseIntError>>()?
                ))
            }
            "320" => Err(FileRequestError::NoSuchFile),
            code @ _ => Err(AniDbError::from((code, reply)).into())
        }
    }
}
