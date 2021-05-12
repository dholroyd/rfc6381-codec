//! Support for codec parameter values
//!
//! See also,
//!  - [MDN: The "codecs" parameter in common media types](https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter)
//!
//! ## Basic usage
//!
//! Parse a codec string,
//! ```rust
//! # use rfc6381_codec::Codec;
//! # use std::str::FromStr;
//! let codec = Codec::from_str("avc1.4D401E");
//! if let Ok(Codec::Avc1(avc1)) = codec {
//!     assert_eq!(avc1.profile(), 0x4d);
//! } else {
//!     panic!("unexpected codec type");
//! }
//! ```
//!
//! Generate a codec string,
//!
//! ```rust
//! # use rfc6381_codec::Codec;
//! let codec = Codec::avc1(0x4d, 0x40, 0x1e);
//! assert_eq!(codec.to_string(), "avc1.4D401E")
//! ```
//!
//! ## No support for 'fancy' syntax
//!
//! RFC 6381 specifies the following BNF grammar for general syntax, which this crate does not
//! yet fully support:
//!
//! ```text
//!   codecs      := cod-simple / cod-fancy
//!   cod-simple  := "codecs" "=" unencodedv
//!   unencodedv  := id-simple / simp-list
//!   simp-list   := DQUOTE id-simple *( "," id-simple ) DQUOTE
//!   id-simple   := element
//!               ; "." reserved as hierarchy delimiter
//!   element     := 1*octet-sim
//!   octet-sim   := <any TOKEN character>
//!
//!               ; Within a 'codecs' parameter value, "." is reserved
//!               ; as a hierarchy delimiter
//!   cod-fancy   := "codecs*" "=" encodedv
//!   encodedv    := fancy-sing / fancy-list
//!   fancy-sing  := [charset] "'" [language] "'" id-encoded
//!               ; Parsers MAY ignore <language>
//!               ; Parsers MAY support only US-ASCII and UTF-8
//!   fancy-list  := DQUOTE [charset] "'" [language] "'" id-list DQUOTE
//!               ; Parsers MAY ignore <language>
//!               ; Parsers MAY support only US-ASCII and UTF-8
//!   id-list     := id-encoded *( "," id-encoded )
//!   id-encoded  := encoded-elm *( "." encoded-elm )
//!               ; "." reserved as hierarchy delimiter
//!   encoded-elm := 1*octet-fancy
//!   octet-fancy := ext-octet / attribute-char
//!
//!   DQUOTE      := %x22 ; " (double quote)
//! ```
//!
//! In particular note the following productions:
//!
//!  - `cod-simple` - specifies the attribute name+value structure `codec=".."` — this crate only
//!    supports dealing with the value of this attribute (the bit inside quotes).
//!  - `cod-fancy` (and related productions `fancy-sing` / `fancy-list` etc.) — show extended
//!    structures that can optionally specify a charset for the data like `en-gb'UTF-8'%25%20xz` or `''%25%20xz` — this crate does not support values
//!    using these structures.

use four_cc::FourCC;
use mp4ra_rust::{ObjectTypeIdentifier, SampleEntryCode};
use mpeg4_audio_const::AudioObjectType;
use std::convert::TryFrom;
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
            level,
        })
    }
}
impl FromStr for Codec {
    type Err = CodecError;

    fn from_str(codec: &str) -> Result<Codec, Self::Err> {
        if let Some(pos) = codec.find('.') {
            let (fourcc, rest) = codec.split_at(pos);
            if fourcc.len() != 4 {
                return Ok(Codec::Unknown(codec.to_string()));
            }
            let fourcc = FourCC::from(fourcc.as_bytes());
            let sample_entry = SampleEntryCode::from(fourcc);
            match sample_entry {
                SampleEntryCode::MP4A => Ok(Codec::Mp4a(get_rest(rest)?.parse()?)),
                SampleEntryCode::AVC1 => Ok(Codec::Avc1(get_rest(rest)?.parse()?)),
                _ => Ok(Codec::Unknown(codec.to_owned())),
            }
        } else {
            Err(CodecError::ExpectedHierarchySeparator(codec.to_string()))
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
    /// The given codec-string-component was not valid
    InvalidComponent(String),
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
impl Avc1 {
    pub fn profile(&self) -> u8 {
        self.profile
    }
    pub fn constraints(&self) -> u8 {
        self.constraints
    }
    pub fn level(&self) -> u8 {
        self.level
    }
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
            .map_err(|_| CodecError::InvalidComponent(value.to_string()))?;

        let constraints = u8::from_str_radix(&value[2..4], 16)
            .map_err(|_| CodecError::InvalidComponent(value.to_string()))?;

        let level = u8::from_str_radix(&value[4..6], 16)
            .map_err(|_| CodecError::InvalidComponent(value.to_string()))?;

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
        audio_object_type: Option<AudioObjectType>,
    },
    Unknown {
        object_type_indication: ObjectTypeIdentifier,
        audio_object_type_indication: Option<u8>,
    },
}
impl fmt::Display for Mp4a {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mp4a::Mpeg4Audio { audio_object_type } => {
                write!(
                    f,
                    "{:02x}",
                    u8::from(ObjectTypeIdentifier::AUDIO_ISO_IEC_14496_3)
                )?;
                if let Some(aoti) = audio_object_type {
                    write!(f, ".{}", u8::from(*aoti))?;
                }
                Ok(())
            }
            Mp4a::Unknown {
                object_type_indication,
                audio_object_type_indication,
            } => {
                write!(f, "{:02x}", u8::from(*object_type_indication))?;
                if let Some(aoti) = audio_object_type_indication {
                    write!(f, ".{}", aoti)?;
                }
                Ok(())
            }
        }
    }
}

impl FromStr for Mp4a {
    type Err = CodecError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut i = value.splitn(2, '.');
        let s = i.next().unwrap();
        let oti =
            u8::from_str_radix(s, 16).map_err(|_| CodecError::InvalidComponent(s.to_string()))?;
        let oti = ObjectTypeIdentifier::from(oti);
        let aoti = i
            .next()
            .map(|v| u8::from_str(v))
            .transpose()
            .map_err(|e| CodecError::InvalidComponent(e.to_string()))?;
        match oti {
            ObjectTypeIdentifier::AUDIO_ISO_IEC_14496_3 => {
                let aoti = aoti
                    .map(AudioObjectType::try_from)
                    .transpose()
                    .map_err(|_e| CodecError::InvalidComponent(aoti.unwrap().to_string()))?;
                Ok(Mp4a::Mpeg4Audio {
                    audio_object_type: aoti,
                })
            }
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
                audio_object_type: Some(AudioObjectType::AAC_SSR)
            }))
        );
        roundtrip("mp4a.40.3");
    }

    #[test]
    fn unknown_oti() {
        const RESERVED_X41: ObjectTypeIdentifier = ObjectTypeIdentifier(0x41);
        assert_matches!(
            Codec::from_str("mp4a.41"),
            Ok(Codec::Mp4a(Mp4a::Unknown {
                object_type_indication: RESERVED_X41,
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
                audio_object_type: Some(AudioObjectType::AAC_LC)
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

    #[test]
    fn invalid_unicode_boundary() {
        // byte position 4 is in the middle of a unicode codepoint - if we naively split off the
        // first 4 bytes this would panic.  We shouldn't panic, we should instead produce an Err.
        assert!(Codec::from_str("cod👍ec").is_err())
    }
}
