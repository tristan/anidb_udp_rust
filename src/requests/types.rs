use crate::mask::FieldDecoder;

#[derive(Debug)]
pub enum EpNo {
    Regular(i32),
    Special(i32),
    Credit(i32),
    Trailer(i32),
    Parody(i32),
    Other(i32),
}

impl EpNo {
    pub fn epno(&self) -> i32 {
        match self {
            EpNo::Regular(x) |
            EpNo::Special(x) |
            EpNo::Credit(x) |
            EpNo::Trailer(x) |
            EpNo::Parody(x) |
            EpNo::Other(x) => *x
        }
    }
}

impl FieldDecoder for EpNo {
    fn decode_field<'a>(
        input: &'a str
    ) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        match &input[..1] {
            "S" => Ok(EpNo::Special(input[1..].parse()?)),
            "C" => Ok(EpNo::Credit(input[1..].parse()?)),
            "T" => Ok(EpNo::Trailer(input[1..].parse()?)),
            "P" => Ok(EpNo::Parody(input[1..].parse()?)),
            "O" => Ok(EpNo::Other(input[1..].parse()?)),
            _ => Ok(EpNo::Regular(input.parse()?))
        }
    }
}
