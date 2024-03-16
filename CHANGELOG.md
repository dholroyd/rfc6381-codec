# Change Log

## Unreleased

### Changed
 - Updated to Rust 2021 Edition

## 0.1.0 - 2021-05-12

### Fixed

 - Avoid panic attempting to parse a codec string with a multi-byte UTF-8 character straddling the position where
   we had expected the initial four-cc substring to end.