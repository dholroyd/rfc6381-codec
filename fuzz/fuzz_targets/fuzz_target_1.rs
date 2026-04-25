#![no_main]
use libfuzzer_sys::fuzz_target;
use rfc6381_codec::Codec;
use std::str::FromStr;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(codec) = Codec::from_str(s) {
            let printed = codec.to_string();
            let reparsed = Codec::from_str(&printed).expect("printed form must re-parse");
            assert_eq!(codec, reparsed, "roundtrip mismatch: input {:?}, printed {:?}", s, printed);
        }
    }
});
