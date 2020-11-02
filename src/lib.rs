//! # String reader
//!
//! Zero-allocation string reader. The string reader can be used to parse
//! all kinds of values from strings. It can be used for construction of
//! traditional lexical analyzers for example. It is useful in situation when
//! you need to parse simple formatted strings but regular expressions are too
//! heavy-weight.
//!
//! # Example
//!
//! Parsing HTTP response header:
//!
//! ```
//! use std::num::ParseIntError;
//!
//! use str_reader::{ParseError, StringReader};
//!
//! /// Parse the first line of an HTTP response header.
//! fn parse_http_response_line(line: &str) -> Result<(u16, &str), HttpParseError> {
//!     let mut reader = StringReader::new(line);
//!
//!     reader.match_str("HTTP/")?;
//!
//!     match reader.read_word() {
//!         "1.0" => (),
//!         "1.1" => (),
//!         _ => return Err(HttpParseError),
//!     }
//!
//!     let status_code = reader.read_u16()?;
//!
//!     Ok((status_code, reader.as_str().trim()))
//! }
//!
//! #[derive(Debug)]
//! struct HttpParseError;
//!
//! impl From<ParseError> for HttpParseError {
//!     fn from(_: ParseError) -> Self {
//!         Self
//!     }
//! }
//!
//! impl From<ParseIntError> for HttpParseError {
//!     fn from(_: ParseIntError) -> Self {
//!         Self
//!     }
//! }
//!
//! let (status_code, status_msg) = parse_http_response_line("HTTP/1.1 404 Not Found").unwrap();
//!
//! assert_eq!(status_code, 404);
//! assert_eq!(status_msg, "Not Found");
//! ```

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
    result,
    str::{Chars, FromStr},
};

/// String reader error.
#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    EmptyInput,
    NoMatch,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
        let msg = match *self {
            Self::EmptyInput => "input is empty",
            Self::NoMatch => "the input does not match",
        };

        f.write_str(msg)
    }
}

impl Error for ParseError {}

/// String reader.
pub struct StringReader<'a> {
    input: Chars<'a>,
    current: Option<char>,
}

impl<'a> StringReader<'a> {
    /// Create a new reader for a given input.
    ///
    /// # Arguments
    ///
    /// * `input` - input string or an object that can be referenced as a
    ///   string
    pub fn new<T>(input: &'a T) -> Self
    where
        T: AsRef<str> + ?Sized,
    {
        let input = input.as_ref().chars();

        // We do not want to advance the input just yet. If we did that the
        // string matching methods would not work.
        let current = input.clone().next();

        Self { input, current }
    }

    /// Get the current character (if any) without advancing the input.
    pub fn current_char(&self) -> Option<char> {
        self.current
    }

    /// Get the next character or return an error if the input is empty.
    pub fn read_char(&mut self) -> Result<char, ParseError> {
        let res = self.input.next().ok_or(ParseError::EmptyInput)?;

        // Peek for the next character without advancing the input.
        self.current = self.input.clone().next();

        Ok(res)
    }

    /// Match a given character to the input and, if successful, advance the
    /// input by exactly one character. An error is returned if the input
    /// character does not match with the given one or if the input is empty.
    ///
    /// # Arguments
    ///
    /// * `expected` - expected character
    pub fn match_char(&mut self, expected: char) -> Result<(), ParseError> {
        let c = self.current_char().ok_or(ParseError::EmptyInput)?;

        if c != expected {
            return Err(ParseError::NoMatch);
        }

        self.skip_char();

        Ok(())
    }

    /// Skip one character.
    pub fn skip_char(&mut self) {
        // Remove the current character.
        self.input.next();

        // Peek for the next character without advancing the input.
        self.current = self.input.clone().next();
    }

    /// Skip all whitespace characters.
    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.skip_char();
            } else {
                break;
            }
        }
    }

    /// Match a given string to the input and, if successful, advance the input
    /// by the length of the given string. An error is returned if the input
    /// does not start with the given string.
    ///
    /// # Arguments
    ///
    /// * `val` - expected string
    pub fn match_str(&mut self, val: &str) -> Result<(), ParseError> {
        let input = self.input.as_str();

        if input.starts_with(val) {
            let (_, rest) = input.split_at(val.len());

            self.input = rest.chars();

            // Peek for the next character without advancing the input.
            self.current = self.input.clone().next();

            Ok(())
        } else {
            Err(ParseError::NoMatch)
        }
    }

    /// Read until a given condition is true or until the end of the input and
    /// return the string.
    ///
    /// # Arguments
    ///
    /// * `cnd` - a closure that takes a single character and returns
    /// true/false
    pub fn read_until<F>(&mut self, cnd: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        let rest = self.input.as_str();

        let index = rest.find(cnd).unwrap_or_else(|| rest.len());

        let (word, rest) = rest.split_at(index);

        self.input = rest.chars();

        // Peek for the next character without advancing the input.
        self.current = self.input.clone().next();

        word
    }

    /// Read one word from the input and return it. A word ends with the first
    /// whitespace character or with the end of the input. The method skips all
    /// initial whitespace characters (if any).
    pub fn read_word(&mut self) -> &'a str {
        self.skip_whitespace();
        self.read_until(char::is_whitespace)
    }

    /// Read the next word and parse it. The input won't be advanced if the
    /// word cannot be parsed.
    pub fn parse_word<T>(&mut self) -> Result<T, T::Err>
    where
        T: FromStr,
    {
        let rest = self.input.as_str().trim_start();

        let index = rest.find(char::is_whitespace).unwrap_or_else(|| rest.len());

        let (word, rest) = rest.split_at(index);

        let parsed = word.parse()?;

        self.input = rest.chars();

        // Peek for the next character without advancing the input.
        self.current = self.input.clone().next();

        Ok(parsed)
    }

    /// Read a decimal integer as i8.
    pub fn read_i8(&mut self) -> Result<i8, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as u8.
    pub fn read_u8(&mut self) -> Result<u8, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as i16.
    pub fn read_i16(&mut self) -> Result<i16, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as u16.
    pub fn read_u16(&mut self) -> Result<u16, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as i32.
    pub fn read_i32(&mut self) -> Result<i32, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as u32.
    pub fn read_u32(&mut self) -> Result<u32, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as i64.
    pub fn read_i64(&mut self) -> Result<i64, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as u64.
    pub fn read_u64(&mut self) -> Result<u64, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as i128.
    pub fn read_i128(&mut self) -> Result<i128, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as u128.
    pub fn read_u128(&mut self) -> Result<u128, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as isize.
    pub fn read_isize(&mut self) -> Result<isize, ParseIntError> {
        self.parse_word()
    }

    /// Read a decimal integer as usize.
    pub fn read_usize(&mut self) -> Result<usize, ParseIntError> {
        self.parse_word()
    }

    /// Read a floating point number as f32.
    pub fn read_f32(&mut self) -> Result<f32, ParseFloatError> {
        self.parse_word()
    }

    /// Read a floating point number as f64.
    pub fn read_f64(&mut self) -> Result<f64, ParseFloatError> {
        self.parse_word()
    }

    /// Check if the reader is empty.
    pub fn is_empty(&self) -> bool {
        self.current_char().is_none()
    }

    /// Get the rest of the input.
    pub fn as_str(&self) -> &'a str {
        self.input.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::StringReader;

    #[test]
    fn test_reader() {
        let input = "Hello, World!!!   1234\n\tfoo-bar";

        let mut reader = StringReader::new(input);

        assert!(!reader.is_empty());
        assert_eq!(reader.current_char(), Some('H'));
        assert_eq!(reader.as_str(), input);

        let word = reader.read_word();

        assert_eq!(word, "Hello,");
        assert_eq!(reader.as_str(), " World!!!   1234\n\tfoo-bar");

        reader.skip_whitespace();

        assert_eq!(reader.as_str(), "World!!!   1234\n\tfoo-bar");

        let c = reader.read_char();

        assert_eq!(c.ok(), Some('W'));

        let res = reader.match_char('o');

        assert!(res.is_ok());

        let res = reader.match_char('R');

        assert!(res.is_err());

        let res = reader.match_str("RLD!!!");

        assert!(res.is_err());

        let res = reader.match_str("rld!!!");

        assert!(res.is_ok());
        assert_eq!(reader.as_str(), "   1234\n\tfoo-bar");

        let n = reader.read_u32();

        assert_eq!(n.ok(), Some(1234));
        assert_eq!(reader.as_str(), "\n\tfoo-bar");

        let n = reader.read_u32();

        assert!(n.is_err());
        assert_eq!(reader.as_str(), "\n\tfoo-bar");

        let word = reader.read_word();

        assert_eq!(word, "foo-bar");
        assert_eq!(reader.as_str(), "");
        assert!(reader.is_empty());

        let word = reader.read_word();

        assert_eq!(word, "");

        let c = reader.read_char();

        assert!(c.is_err());
        assert!(reader.is_empty());
        assert_eq!(reader.as_str(), "");
    }
}
