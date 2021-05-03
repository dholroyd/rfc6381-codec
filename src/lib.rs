use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
#[non_exhaustive]
pub enum Codec {
    Avc1(Avc1),
    Mp4a(Mp4a),
    Unknown(String),
}
impl Codec {
    pub fn parse_codecs(codecs: &str) -> impl Iterator<Item = Result<Codec, CodecError>> + '_ {
        codecs.split(',').map(|s| s.trim().parse())
    }

    pub fn avc1(profile: u8, constraints: u8, level: u8) -> Self {
        Codec::Avc1(Avc1 {
            profile,
            constraints,
            level
        })
    }
}
impl FromStr for Codec {
    type Err = CodecError;

    fn from_str(codec: &str) -> Result<Codec, Self::Err> {
        if codec.len() < 4 {
            Ok(Codec::Unknown(codec.to_owned()))
        } else {
            let (fourcc, rest) = codec.split_at(4);
            match fourcc {
                "mp4a" => Ok(Codec::Mp4a(get_rest(rest)?.parse()?)),
                "avc1" => Ok(Codec::Avc1(get_rest(rest)?.parse()?)),
                _ => Ok(Codec::Unknown(codec.to_owned())),
            }
        }
    }
}
impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Codec::Avc1(Avc1 {
                profile,
                constraints,
                level,
            }) => write!(f, "avc1.{:02X}{:02X}{:02X}", profile, constraints, level),
            Codec::Mp4a(mp4a) => write!(f, "mp4a.{}", mp4a),
            Codec::Unknown(val) => f.write_str(val),
        }
    }
}

fn get_rest(text: &str) -> Result<&str, CodecError> {
    if text.is_empty() {
        Ok(text)
    } else if let Some(rest) = text.strip_prefix('.') {
        Ok(rest)
    } else {
        Err(CodecError::ExpectedHierarchySeparator(text.to_string()))
    }
}

#[derive(Debug)]
pub enum CodecError {
    /// The given string could not be interpreted as hexadecimal digits of the expected value-size
    InvalidHex(String),
    /// expected the '.', but instead found the text included in the variant
    ExpectedHierarchySeparator(String),
    /// The length of the given string did not match the expected length
    UnexpectedLength { expected: usize, got: String },
}

#[derive(Debug)]
pub struct Avc1 {
    profile: u8,
    constraints: u8,
    level: u8,
}

impl FromStr for Avc1 {
    type Err = CodecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 6 {
            return Err(CodecError::UnexpectedLength {
                expected: 6,
                got: value.to_string(),
            });
        }

        let profile = u8::from_str_radix(&value[0..2], 16)
            .map_err(|_| CodecError::InvalidHex(value.to_string()))?;

        let constraints = u8::from_str_radix(&value[2..4], 16)
            .map_err(|_| CodecError::InvalidHex(value.to_string()))?;

        let level = u8::from_str_radix(&value[4..6], 16)
            .map_err(|_| CodecError::InvalidHex(value.to_string()))?;

        Ok(Avc1 {
            profile,
            constraints,
            level,
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Mp4a {
    Mpeg4Audio {
        audio_object_type: Option<u8>,
    },
    Unknown {
        object_type_indication: u8,
        audio_object_type_indication: Option<u8>,
    },
}
impl fmt::Display for Mp4a {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mp4a::Mpeg4Audio { audio_object_type } => {
                if let Some(aoti) = audio_object_type {
                    write!(f, "40.{:x}", aoti)
                } else {
                    write!(f, "40")
                }
            }
            Mp4a::Unknown {
                object_type_indication,
                audio_object_type_indication,
            } => {
                if let Some(aoti) = audio_object_type_indication {
                    write!(f, "{:02x}.{:x}", object_type_indication, aoti)
                } else {
                    write!(f, "{:02x}", object_type_indication)
                }
            }
        }
    }
}

impl FromStr for Mp4a {
    type Err = CodecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut i = value.splitn(2, '.');
        let s = i.next().unwrap();
        let oti = u8::from_str_radix(s, 16).map_err(|_| CodecError::InvalidHex(s.to_string()))?;
        let aoti = i
            .next()
            .map(|v| u8::from_str_radix(v, 16))
            .transpose()
            .map_err(|e| CodecError::InvalidHex(e.to_string()))?;
        match oti {
            0x40 => Ok(Mp4a::Mpeg4Audio {
                audio_object_type: aoti,
            }),
            _ => Ok(Mp4a::Unknown {
                object_type_indication: oti,
                audio_object_type_indication: aoti,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::*;

    fn roundtrip(codec: &str) {
        assert_eq!(codec, Codec::from_str(codec).unwrap().to_string())
    }

    #[test]
    fn mp4a() {
        assert_matches!(
            Codec::from_str("mp4a.40.3"),
            Ok(Codec::Mp4a(Mp4a::Mpeg4Audio {
                audio_object_type: Some(3)
            }))
        );
        roundtrip("mp4a.40.3");
    }

    #[test]
    fn unknown_oti() {
        assert_matches!(
            Codec::from_str("mp4a.41"),
            Ok(Codec::Mp4a(Mp4a::Unknown {
                object_type_indication: 0x41,
                audio_object_type_indication: None
            }))
        );
        roundtrip("mp4a.41");
    }

    #[test]
    fn bad_oti_digit() {
        assert_matches!(Codec::from_str("mp4a.4g"), Err(_));
    }

    #[test]
    fn list() {
        let mut i = Codec::parse_codecs("mp4a.40.2,avc1.4d401e");
        assert_matches!(
            i.next().unwrap(),
            Ok(Codec::Mp4a(Mp4a::Mpeg4Audio {
                audio_object_type: Some(2)
            }))
        );
        assert_matches!(
            i.next().unwrap(),
            Ok(Codec::Avc1(Avc1 {
                profile: 0x4d,
                constraints: 0x40,
                level: 0x1e
            }))
        );
    }

    #[test]
    fn avc1() {
        assert_matches!(
            Codec::from_str("avc1.4d401e"),
            Ok(Codec::Avc1(Avc1 {
                profile: 0x4d,
                constraints: 0x40,
                level: 0x1e
            }))
        );
        roundtrip("avc1.4D401E");
    }

    #[test]
    fn bad_avc1_lengths() {
        assert_matches!(Codec::from_str("avc1.41141"), Err(CodecError::UnexpectedLength { expected: 6, got: text }) if text == "41141");
        assert_matches!(Codec::from_str("avc1.4114134"), Err(CodecError::UnexpectedLength { expected: 6, got: text }) if text == "4114134");
    }

    #[test]
    fn unknown_fourcc() {
        assert_matches!(Codec::from_str("badd.41"), Ok(Codec::Unknown(v)) if v == "badd.41");
        roundtrip("badd.41");
    }
}
