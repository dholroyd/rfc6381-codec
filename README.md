# rfc6381-codec

Rust library for parsing and generating _codec_ string values, as specified in
[RFC 6381, section 3](https://tools.ietf.org/html/rfc6381#section-3).

## Supported RFC 6381 features

 - [x] `avc1`
 - [x] `mp4a` only object-type-identifier `0x40` (MPEG 4 Audio) supported
 - [ ] other four-character-code values not supported
 - [ ] generic syntax including 'charset' and 'percent-encoding' not supported