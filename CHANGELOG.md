# Change Log

## Unreleased

### Added
 - `PartialEq` and `Eq` are now derived for `Codec`, `Avc`, and `Mp4a`.
 - `Codec::Avc3` variant and `Codec::avc3()` constructor for the `avc3` fourcc, which shares the
   `avc1` codec-string grammar (ISO/IEC 14496-15; signals in-band parameter sets).

### Changed
 - Updated to Rust 2024 Edition (requires Rust 1.85 or later).
 - Renamed the `Avc1` struct to `Avc` (it carries the profile/constraints/level triple
   shared by both the `avc1` and `avc3` codec-string forms).

### Fixed
 - Avoid panic parsing an `avc1.` codec string whose 6-byte payload contains a multi-byte UTF-8
   character, which previously caused byte-indexing to land mid-codepoint.

## 0.2.0 - 2024-03-17

### Changed
 - Updated to Rust 2021 Edition

## 0.1.0 - 2021-05-12

### Fixed

 - Avoid panic attempting to parse a codec string with a multi-byte UTF-8 character straddling the position where
   we had expected the initial four-cc substring to end.
