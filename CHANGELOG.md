# ChangeLog

## Unreleased

### Fixed

 - Avoid panic attempting to parse a codec string with a multi-byte UTF-8 character straddling the position where
   we had expected the initial four-cc substring to end.