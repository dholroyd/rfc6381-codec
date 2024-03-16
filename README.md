# rfc6381-codec

Rust library for parsing and generating _codec_ string values, as specified in
[RFC 6381, section 3](https://tools.ietf.org/html/rfc6381#section-3).

[![crates.io version](https://img.shields.io/crates/v/rfc6381-codec.svg)](https://crates.io/crates/rfc6381-codec)
[![Documentation](https://docs.rs/rfc6381-codec/badge.svg)](https://docs.rs/rfc6381-codec)
[![Coverage Status](https://coveralls.io/repos/github/dholroyd/rfc6381-codec/badge.svg?branch=master)](https://coveralls.io/github/dholroyd/rfc6381-codec?branch=master)

## Supported RFC 6381 features

 - [x] `avc1`
 - [x] `mp4a` only object-type-identifier `0x40` (MPEG 4 Audio) supported
 - [ ] other four-character-code values not supported
 - [ ] generic syntax including 'charset' and 'percent-encoding' not supported