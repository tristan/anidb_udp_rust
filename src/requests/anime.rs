use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use super::AniDbRequest;
use crate::errors::AniDbError;

pub enum AnimeRequest {
    Aid(u32, Option<AnimeRequestFields>),
    Aname(String, Option<AnimeRequestFields>)
}

impl AnimeRequest {
    pub fn from_anime_id(aid: u32, fields: Option<AnimeRequestFields>) -> AnimeRequest {
        AnimeRequest::Aid(aid, fields)
    }
    pub fn from_anime_name(aname: &str, fields: Option<AnimeRequestFields>) -> AnimeRequest {
        AnimeRequest::Aname(aname.to_string(), fields)
    }
}

impl AniDbRequest for AnimeRequest {
    type Response = Vec<AnimeResponse>;
    type Error = AnimeRequestError;
    fn name() -> &'static str {
        "ANIME"
    }
    fn decode_response(
        &self,
        code: &str,
        reply: &str,
        data: &str
    ) -> Result<Self::Response, Self::Error> {
        match &code[..] {
            "230" => {
                data.split("\n").into_iter()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| {
                        self.decode_line(line)
                    }).collect()
            },
            "330" => Err(AnimeRequestError::NoSuchAnime),
            code @ _ => Err(AniDbError::from((code, reply)).into())
        }
    }
}

impl AnimeRequest {
    fn decode_line(&self, line: &str) -> Result<AnimeResponse, AnimeRequestError> {
        let resp = match self {
            AnimeRequest::Aid(_, Some(fields)) | AnimeRequest::Aname(_, Some(fields)) => {
                // 15456|0|2021-2021|TV Series|14767|2|Tensei Shitara Slime Datta Ken (2021 Dai 2 Bu)|転生したらスライムだった件 (2021 第2部)|That Time I Got Reincarnated as a Slime (2021 Part 2)|転生したらスライムだった件 (2021 第2部)'That Time I Got Reincarnated as a Slime (2021 Part 2)|Tensura 2 Part 2'Tensura (2021 Part 2)||12|8|3|1625529600|1632182400|http://www.ten-sura.com/|261282.jpg|743|76|743|79|0|0||0|24212|372060||themes,original work,novel|2607,2609,2799|0,0,0|1627893775|96305,98294,98296,98297,98298,98299,99097,99348,99350,99971,99972,101836,101838,101839,101846,114732,114735,114737,114739,118370,118466,118467,118717,118869,118870,118871,119025,119026,96306,96309,96299,96301,96302,96303,96304,96307,97711,97712,97713,97715,99349,101861,114738,117011,117348|1|2|0|0|0
                let mut field_iter = line.split("|");
                fields.decode_response(&mut field_iter)?
            },
            _ => {
                let mut resp = AnimeResponse::default();
                let mut field_iter = line.split("|");
                decode_field!(field_iter, resp, aid, i32);
                decode_field!(field_iter, resp, episodes, i32);
                decode_field!(field_iter, resp, highest_episode_number, i32);
                decode_field!(field_iter, resp, special_ep_count, i32);
                decode_field!(field_iter, resp, rating, i32);
                decode_field!(field_iter, resp, vote_count, i32);
                decode_field!(field_iter, resp, temp_rating, i32);
                decode_field!(field_iter, resp, temp_vote_count, i32);
                decode_field!(field_iter, resp, average_view_rating, i32);
                decode_field!(field_iter, resp, review_count, i32);
                decode_field!(field_iter, resp, year, String);
                decode_field!(field_iter, resp, ty, String);
                decode_field!(field_iter, resp, romaji_name, String);
                decode_field!(field_iter, resp, kanji_name, String);
                decode_field!(field_iter, resp, english_name, String);
                decode_field!(field_iter, resp, other_name, String);
                decode_field!(field_iter, resp, short_name_list, String);
                decode_field!(field_iter, resp, synonym_list, String);
                decode_field!(field_iter, resp, category_list, Vec<String>);
                resp
            }
        };
        Ok(resp)
    }
}

impl Serialize for AnimeRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let len = match self {
            AnimeRequest::Aid(_, Some(_)) | AnimeRequest::Aname(_, Some(_)) => 2,
            _ => 1
        };
        let mut map = serializer.serialize_map(Some(len))?;
        let fields = match self {
            AnimeRequest::Aid(aid, fields) => {
                map.serialize_entry("aid", aid)?;
                fields
            },
            AnimeRequest::Aname(aname, fields) => {
                map.serialize_entry("aname", aname)?;
                fields
            }
        };
        if let Some(fields) = fields {
            map.serialize_entry("amask", fields)?;
        }
        map.end()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AnimeRequestError {
    #[error("NO SUCH ANIME")]
    NoSuchAnime,
    #[error("{0}")]
    AniDbError(#[from] AniDbError)
}
impl_into_anidberror!(AnimeRequestError);

generate_mask!(
    AnimeRequestFields,
    AnimeResponse,
    AnimeRequestError,
    7, i32, aid,
    6, i32, dateflags,
    5, String, year,
    4, String, ty,
    3, String, related_aid_list,
    2, String, related_aid_type
        &
    7, String, romaji_name,
    6, String, kanji_name,
    5, String, english_name,
    4, String, other_name,
    3, String, short_name_list,
    2, String, synonym_list
        &
    7, i32, episodes,
    6, i32, highest_episode_number,
    5, i32, special_ep_count,
    4, i32, air_date,
    3, i32, end_date,
    2, String, url,
    1, String, picname
        &
    7, i32, rating,
    6, i32, vote_count,
    5, i32, temp_rating,
    4, i32, temp_vote_count,
    3, i32, average_view_rating,
    2, i32, review_count,
    1, String, award_list,
    0, bool, is_18plus_restricted
        &
    6, i32, ann_id,
    5, i32, allcinema_id,
    4, String, animenfo_id,
    3, Vec<String>, tag_name_list,
    2, Vec<i32>, tag_id_list,
    1, Vec<i32>, tag_weight_list,
    0, i32, date_record_updated
        &
    7, Vec<i32>, character_id_list
        &
    7, i32, specials_count,
    6, i32, credits_count,
    5, i32, other_count,
    4, i32, trailer_count,
    3, i32, parody_count
        ;
    Vec<String>, category_list
);
