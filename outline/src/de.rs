use crate::outline::Outline;
use lazy_static::lazy_static;
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use std::error;
use std::fmt::{self, Write};
use std::str::FromStr;

// Magic __heading__ field, this means we're deserialing
//
//     title
//       x 1
//       y 2
//
// instead of
//
//       heading title
//       x 1
//       y 2
pub(crate) const MAGIC_HEADING_NAME: &str = "__heading__";

lazy_static! {
    static ref MAGIC_OUTLINE: Outline = {
        Outline {
            headline: Some(MAGIC_HEADING_NAME.into()),
            children: Vec::new(),
        }
    };
}

type Result<T> = std::result::Result<T, Error>;

struct Deserializer<'de> {
    outline: &'de Outline,
    offset: usize,
    is_inline_seq: bool,
}

pub fn from_outline<'de, T: de::Deserialize<'de>>(outline: &'de Outline) -> Result<T> {
    let mut deserializer = Deserializer {
        outline,
        offset: 0,
        is_inline_seq: false,
    };

    let ret = T::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(ret)
}

impl<'de> Deserializer<'de> {
    fn next_token_end(&self) -> Option<usize> {
        if let Some(headline) = &self.outline.headline {
            let s = &headline[self.offset..];
            if s.is_empty() {
                return None;
            }
            // Make sure you consume at least one character if there's trailing space.
            // XXX: Might want to eat all trailing space and whatever non-space comes after that.
            let first_char_width = s.chars().next().unwrap().len_utf8();
            let offset = s[first_char_width..]
                .find(' ')
                .map(|x| x + first_char_width)
                .unwrap_or_else(|| s.len());
            Some(self.offset + offset)
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Result<char> {
        if let Some(headline) = &self.outline.headline {
            if let Some(c) = &headline[self.offset..].chars().next() {
                self.offset += c.len_utf8();
                return Ok(*c);
            }
        }
        Err(Error::default())
    }

    fn headline_len(&self) -> Option<usize> { self.outline.headline.as_ref().map(|s| s.len()) }

    fn peek_token(&self) -> Option<&str> {
        if let (Some(headline), Some(token_end)) = (&self.outline.headline, self.next_token_end()) {
            Some(&headline[self.offset..token_end])
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Option<&str> {
        if let (Some(headline), Some(token_end)) = (&self.outline.headline, self.next_token_end()) {
            let ret = Some(&headline[self.offset..token_end]);
            self.offset = token_end;
            // Skip the one space
            let _ = self.next_char();
            ret
        } else {
            None
        }
    }

    fn parse_next<T: FromStr>(&mut self) -> Result<T> {
        if let Some(tok) = self.peek_token() {
            if let Ok(val) = tok.parse() {
                self.next_token();
                return Ok(val);
            }
        }
        Err(Error::default())
    }

    fn headline_tail(&self) -> Option<&str> {
        if let Some(headline) = &self.outline.headline {
            if self.offset < headline.len() {
                return Some(&headline[self.offset..]);
            }
        }
        None
    }

    fn set_fully_consumed(&mut self) {
        while !self.outline.children.is_empty() {
            let last_idx = self.outline.children.len() - 1;
            self.outline = &self.outline.children[last_idx];
        }
        self.offset = self.headline_len().unwrap_or(0);
    }

    fn parse_string(&mut self) -> Result<String> {
        if let Some(tail) = self.headline_tail() {
            if self.is_inline_seq {
                // If currently in sequence, strings are whitespace-separated
                self.parse_next()
            } else {
                // Otherwise string is to the end of input
                let new_offset = self.headline_len().unwrap_or(0);
                let ret: String = tail.into();
                self.offset = new_offset;
                Ok(ret)
            }
        } else if !self.outline.children.is_empty() {
            // No more headline left, read children as the string literal.
            let mut ret = String::new();
            for c in &self.outline.children {
                let _ = write!(&mut ret, "{}", c);
            }
            self.set_fully_consumed();
            Ok(ret)
        } else {
            Err(Error::default())
        }
    }

    /// Check that all data has been consumed.
    fn end(&self) -> Result<()> {
        if !self.outline.children.is_empty() {
            return Err(Error::default());
        }
        if self.headline_tail().is_some() {
            return Err(Error::default());
        }
        Ok(())
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // This is limited since the data format is not self-describing.
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    // Primitive types just use the default FromStr behavior

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.parse_next()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse_next()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse_next()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse_next()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse_next()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse_next()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse_next()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse_next()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse_next()?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.parse_next()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.parse_next()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // XXX: Should this just be unimplemented and you should use string parse insted?
        if let Some(token) = self.peek_token() {
            if token.chars().count() == 1 {
                let token = self.next_token().unwrap();
                return visitor.visit_char(token.chars().next().unwrap());
            }
        }
        return Err(Error::default());
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // XXX: No way currently to express an explicit None in data.
        // Options are expected to be used in structs and by omitting the whole struct field from
        // the literal.
        //
        // Maybe a dedicated 'nil' literal could be introduced if we really need this?
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.is_inline_seq {
            // Double nesting detected
            return Err(Error::default());
        }

        let seq = if self.headline_tail().is_some() {
            self.is_inline_seq = true;
            Sequence {
                de: self,
                cursor: Cursor::Inline,
            }
        } else {
            Sequence {
                de: self,
                cursor: Cursor::Child(0, 0),
            }
        };

        let ret = visitor.visit_seq(seq);
        if !self.is_inline_seq {
            // Ate all the children
            self.set_fully_consumed();
        }

        self.is_inline_seq = false;
        ret
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // XXX: Repetition shared with deserialize_seq, factor out?
        if self.is_inline_seq {
            // Double nesting detected
            return Err(Error::default());
        }

        let seq = if self.headline_tail().is_some() {
            self.is_inline_seq = true;
            Sequence {
                de: self,
                cursor: Cursor::Inline,
            }
        } else {
            Sequence {
                de: self,
                cursor: Cursor::Child(0, 0),
            }
        };

        let ret = visitor.visit_map(seq);
        if !self.is_inline_seq {
            // Ate all the children
            self.set_fully_consumed();
        }

        self.is_inline_seq = false;
        ret
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if fields.contains(&MAGIC_HEADING_NAME) {
            if self.is_inline_seq {
                // Double nesting detected
                return Err(Error::default());
            }
            let seq = Sequence {
                de: self,
                cursor: Cursor::MagicHeading,
            };

            let ret = visitor.visit_map(seq);
            self.set_fully_consumed();
            ret
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// Cursor for progressing through a map or a seq.
enum Cursor {
    /// Cursor for inline data, use deserializer's position
    Inline,
    /// Cursor for vertical data, parameters are nth child, kth offset in headline
    Child(usize, usize),
    /// Cursor in magic headline of vertical data.
    ///
    /// After parsing the headline into magic heading field name, the cursor will change into
    /// Child(0, 0) and start parsing vertical data.
    MagicHeading,
}

/// Sequence accessor for items in a single line.
///
/// Uses whitespace as separator, string values in an inline list cannot have whitespace.
struct Sequence<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    cursor: Cursor,
}

impl<'a, 'de> de::SeqAccess<'de> for Sequence<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.cursor {
            Cursor::Inline => {
                if self.de.headline_tail().is_none() {
                    Ok(None)
                } else {
                    seed.deserialize(&mut *self.de).map(Some)
                }
            }
            Cursor::Child(n, offset) => {
                if n >= self.de.outline.children.len() {
                    Ok(None)
                } else {
                    let mut child_de = Deserializer {
                        outline: &self.de.outline.children[n],
                        offset,
                        is_inline_seq: false,
                    };
                    self.cursor = Cursor::Child(n + 1, 0);
                    seed.deserialize(&mut child_de).map(Some)
                }
            }
            Cursor::MagicHeading => {
                // Turn the cursor into a regular vertical data cursor.
                self.cursor = Cursor::Child(0, 0);
                // Deserialize actual contents of the heading line into the value.
                seed.deserialize(&mut *self.de).map(Some)
            }
        }
    }
}

impl<'a, 'de> de::MapAccess<'de> for Sequence<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.cursor {
            Cursor::Inline => {
                if self.de.headline_tail().is_none() {
                    Ok(None)
                } else {
                    seed.deserialize(&mut *self.de).map(Some)
                }
            }
            Cursor::Child(n, offset) => {
                if n >= self.de.outline.children.len() {
                    Ok(None)
                } else {
                    let mut child_de = Deserializer {
                        outline: &self.de.outline.children[n],
                        offset: offset,
                        is_inline_seq: true,
                    };
                    let ret = seed.deserialize(&mut child_de).map(Some);
                    // Save parse offset from key
                    // XXX: keys must always be inline values
                    self.cursor = Cursor::Child(n, child_de.offset);
                    ret
                }
            }
            Cursor::MagicHeading => {
                // Create the magic heading key name out of nowhere.
                let mut temp_de = Deserializer {
                    outline: &MAGIC_OUTLINE,
                    offset: 0,
                    is_inline_seq: true,
                };
                seed.deserialize(&mut temp_de).map(Some)
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        //self.cursor = Cursor::Child(n + 1);
        match self.cursor {
            Cursor::Inline => seed.deserialize(&mut *self.de),
            Cursor::Child(n, offset) => {
                // TODO: Figure this out!!!
                // Need to apply parse after key is taken
                let mut child_de = Deserializer {
                    outline: &self.de.outline.children[n],
                    offset,
                    is_inline_seq: false,
                };
                self.cursor = Cursor::Child(n + 1, 0);
                let ret = seed.deserialize(&mut child_de);
                child_de.end()?;
                ret
            }
            Cursor::MagicHeading => {
                self.cursor = Cursor::Child(0, 0);
                seed.deserialize(&mut *self.de)
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Error(String);

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error { Error(format!("{}", msg)) }
}

impl error::Error for Error {
    fn description(&self) -> &str { &self.0 }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

#[cfg(test)]
mod de_tests {
    use super::*;
    use crate::outline::Outline;

    #[test]
    fn test_tokenizer() {
        let mut outline = Outline::from("foo bar baz");
        outline.lift_singleton();
        let mut de = Deserializer {
            outline: &outline,
            offset: 0,
            is_inline_seq: false,
        };

        assert_eq!(de.peek_token(), Some("foo"));
        assert_eq!(de.next_token(), Some("foo"));
        assert_eq!(de.next_token(), Some("bar"));
        assert_eq!(de.next_token(), Some("baz"));
    }
}
