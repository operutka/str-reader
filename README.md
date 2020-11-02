# String reader

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][license-badge]][license-url]
[![Build Status][build-badge]][build-url]

[crates-badge]: https://img.shields.io/crates/v/str-reader
[crates-url]: https://crates.io/crates/str-reader
[license-badge]: https://img.shields.io/crates/l/str-reader
[license-url]: https://github.com/operutka/str-reader/blob/master/LICENSE
[build-badge]: https://travis-ci.org/operutka/str-reader.svg?branch=master
[build-url]: https://travis-ci.org/operutka/str-reader

Zero-allocation string reader. The string reader can be used to parse
all kinds of values from strings. It can be used for construction of
traditional lexical analyzers for example. It is useful in situation when
you need to parse simple formatted strings but regular expressions are too
heavy-weight.

## Example

Parsing HTTP response header:

```rust
use std::num::ParseIntError;

use str_reader::{ParseError, StringReader};

/// Parse the first line of an HTTP response header.
fn parse_http_response_line(line: &str) -> Result<(u16, &str), HttpParseError> {
    let mut reader = StringReader::new(line);

    reader.match_str("HTTP/")?;

    match reader.read_word() {
        "1.0" => (),
        "1.1" => (),
        _ => return Err(HttpParseError),
    }

    let status_code = reader.read_u16()?;

    Ok((status_code, reader.as_str().trim()))
}

#[derive(Debug)]
struct HttpParseError;

impl From<ParseError> for HttpParseError {
    fn from(_: ParseError) -> Self {
        Self
    }
}

impl From<ParseIntError> for HttpParseError {
    fn from(_: ParseIntError) -> Self {
        Self
    }
}

let (status_code, status_msg) = parse_http_response_line("HTTP/1.1 404 Not Found").unwrap();

assert_eq!(status_code, 404);
assert_eq!(status_msg, "Not Found");
```
