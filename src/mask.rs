macro_rules! next_or_decode_error {
    ($iter:ident) => {
        $iter.next()
            .ok_or_else(|| crate::AniDbError::DecodeError(String::from("Missing expected field")))
    }
}

pub trait FieldDecoder {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
        where Self: Sized;
}

impl FieldDecoder for i32 {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(input.parse()?)
    }
}

impl FieldDecoder for i16 {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(input.parse()?)
    }
}

impl FieldDecoder for i64 {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(input.parse()?)
    }
}

impl FieldDecoder for String {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(input.to_string())
    }
}

impl FieldDecoder for bool {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(<i16 as FieldDecoder>::decode_field(input)? == 1)
    }
}

impl<T> FieldDecoder for Vec<T> where T: FieldDecoder {
    fn decode_field<'a>(input: &'a str) -> Result<Self, crate::AniDbError>
    where Self: Sized {
        Ok(input.split(",")
            .map(|part| <T as FieldDecoder>::decode_field(part))
            .collect::<Result<Vec<T>, _>>()?)
    }
}

macro_rules! decode_field {
    ($iter:ident, $resp:ident, $field:tt, $ty:ty) => {
        let $field = <$ty as crate::mask::FieldDecoder>::decode_field(next_or_decode_error!($iter)?)?;
        $resp.$field = Some($field);
    };
    ($iter:ident, $mask:ident, $resp:ident, $field:tt, $ty:ty) => {
        if $mask.$field {
            let $field = <$ty as crate::mask::FieldDecoder>::decode_field(next_or_decode_error!($iter)?)?;
            $resp.$field = Some($field);
        }
    }
}

macro_rules! generate_mask {
    ($builder:ident, $resp:ident, $decode_err:ident, $($($offset:expr, $ty:ty, $name:ident),*)&*;$($ety:ty, $ename:ident)*) => {
        #[derive(Clone, Default, ::typed_builder::TypedBuilder)]
        pub struct $builder {
            $(
                $(
                    #[builder(default)]
                    $name: bool,
                )*
            )*
        }

        impl $builder {
            pub fn all() -> $builder {
                $builder {
                    $(
                        $(
                            $name: true,
                        )*
                    )*
                }
            }
            pub fn none() -> $builder {
                $builder {
                    $(
                        $(
                            $name: false,
                        )*
                    )*
                }
            }
        }

        impl ::serde::Serialize for $builder {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: ::serde::Serializer {
                let mut bytes = Vec::new();
                $(
                    let mut b = 0;
                    $(
                        b += (self.$name as u8).overflowing_shl($offset).0;
                    )*
                    bytes.push(b);
                )*
                let s = ::hex::encode(bytes);
                serializer.serialize_str(&s)
            }
        }

        #[derive(Default, Debug)]
        pub struct $resp {
            $(
                $(
                    pub $name: Option<$ty>,
                )*
            )*
            $(
                pub $ename: Option<$ety>,
            )*
        }

        impl $builder {
            fn decode_response<'a>(
                &self,
                field_iter: &mut impl Iterator<Item = &'a str>
            ) -> Result<$resp, $decode_err> {
                let mut resp = $resp::default();
                $(
                    $(
                        decode_field!(field_iter, self, resp, $name, $ty);
                    )*
                )*
                Ok(resp)
            }
        }

    }
}
