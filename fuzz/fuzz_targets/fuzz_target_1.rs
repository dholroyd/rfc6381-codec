#![no_main]
use libfuzzer_sys::fuzz_target;
use rfc6381_codec::Codec;
use std::str::FromStr;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(codec) = Codec::from_str(s) {
            assert_eq!(codec.to_string(), s);
        }
    }
});
