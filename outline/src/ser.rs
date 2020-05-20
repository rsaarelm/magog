use crate::de::MAGIC_HEADING_NAME;
use crate::outline::Outline;
use serde::{ser, Deserialize, Serialize};
use std::convert::TryFrom;
use std::error;
use std::fmt;

/// Maximum line length for inlined compound expressions.
const MAX_INLINE_EXPRESSION_LENGTH: usize = 80;

type Result<T> = std::result::Result<T, Error>;

/// Descriptor for elements that correspond to a single value.
#[derive(Eq, PartialEq, Clone, Debug)]
enum Value {
    /// Empty value, when serializing None
    Nil,
    /// Value with no whitespace
    Word(String),
    /// Value that contains spaces but no newlines. Usually a String.
    Sentence(String),
    /// Value that contains newlines. Usually a String.
    Paragraph(String),
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum Expr {
    Atom(Value),
    Entry {
        key: Value,
        value: Box<Expr>,
    },
    List(Vec<Expr>),
    Titled {
        heading: Box<Expr>,
        contents: Vec<Expr>,
    },
}

use {Expr::*, Value::*};

pub fn into_outline<T: Serialize>(value: T) -> Result<Outline> {
    let mut serializer = Serializer::default();
    let expr = value.serialize(&mut serializer)?;
    // TODO: Fix error propagation
    //Outline::try_from(expr)?
    Ok(Outline::try_from(expr).unwrap())
}

impl<T: fmt::Display> From<T> for Value {
    fn from(item: T) -> Value { Value::new(format!("{}", item)) }
}

impl Value {
    pub fn new(text: impl Into<String>) -> Value {
        let text: String = text.into();

        if text.is_empty() {
            return Value::Nil;
        }

        let mut newline = false;
        let mut space = false;
        for c in text.chars() {
            if c.is_whitespace() {
                space = true;
            }
            if c == '\n' {
                newline = true;
                // We can't learn anything more, might as well call it quits.
                break;
            }
        }

        if newline {
            Value::Paragraph(text)
        } else if space {
            Value::Sentence(text)
        } else {
            Value::Word(text)
        }
    }

    fn len(&self) -> usize {
        match self {
            Nil => 0,
            Word(s) => s.len(),
            Sentence(s) => s.len(),
            Paragraph(s) => s.len(),
        }
    }
}

impl Expr {
    fn can_be_inlined(&self) -> bool {
        match self {
            Atom(Nil) => true,
            Atom(Word(_)) => true,
            Atom(_) => false,
            Entry {
                key: Word(_),
                value: e,
            } if e.is_symbol() || e.is_empty() => true,
            List(es) if es.iter().all(|e| e.can_be_inlined() && !e.is_list()) => true,
            _ => false,
        }
    }

    fn is_list(&self) -> bool {
        match self {
            List(_) => true,
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Atom(Nil) => true,
            Atom(Word(w)) if w.is_empty() => true,
            List(es) if es.is_empty() => true,
            _ => false,
        }
    }

    fn is_symbol(&self) -> bool {
        match self {
            Atom(Word(_)) => true,
            _ => false,
        }
    }

    fn is_valid(&self) -> Result<()> {
        if self.is_empty() {
            return Err(Error::default());
        }
        match self {
            Atom(_) => Ok(()),
            Entry {
                key: Word(_),
                value: e,
            } => e.is_valid(),
            Entry { .. } => Err(Error::default()), // Keys must be words
            // A list must be either all entries or no entries.
            List(es) => {
                let num_entries = es
                    .iter()
                    .filter(|e| if let Entry { .. } = e { true } else { false })
                    .count();
                if num_entries != 0 && num_entries != es.len() {
                    // A list must be either all entries or all non-entry expressions.
                    Err(Error::default())
                } else if let Some(e) = es.iter().find_map(|e| e.is_valid().err()) {
                    // Invalid sub-item found.
                    Err(e)
                } else {
                    Ok(())
                }
            }
            Titled { heading, contents } => {
                heading.is_valid()?;
                for e in contents {
                    if let Entry { .. } = e {
                    } else {
                        return Err(Error::default());
                    }
                }
                Ok(())
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Atom(v) => v.len(),
            Entry { key, value } => key.len() + 1 + value.len(),
            List(es) if !es.is_empty() => es.iter().map(|e| e.len()).sum::<usize>() + es.len() - 1,
            List(_) => 0,
            Titled { heading, contents } => heading.len() + 1 + contents.len(),
        }
    }
}

impl<T: Into<Value>> From<T> for Expr {
    fn from(item: T) -> Expr { Atom(item.into()) }
}

impl TryFrom<Expr> for Outline {
    type Error = Box<dyn std::error::Error>;

    fn try_from(e: Expr) -> std::result::Result<Outline, Self::Error> {
        e.is_valid()?;
        let is_inlined = e.can_be_inlined() && e.len() < MAX_INLINE_EXPRESSION_LENGTH;

        match e {
            Atom(Nil) => Ok(Default::default()),
            Atom(Word(s)) => Ok(Outline::new(s, vec![])),
            Atom(Sentence(s)) => Ok(Outline::new(s, vec![])),
            Atom(Paragraph(s)) => {
                let mut ret = Outline::default();
                for line in s.lines() {
                    ret.push_str(line);
                }
                Ok(ret)
            }
            Entry { key, value } => {
                // Don't print things with empty value
                if value.is_empty() {
                    Ok(Default::default())
                } else {
                    let mut ret = Outline::try_from(Atom(key))?;
                    ret.concatenate(Outline::try_from(*value)?);
                    Ok(ret)
                }
            }
            List(es) => {
                let mut ret = Outline::default();
                for e in es {
                    if is_inlined {
                        ret.concatenate(Outline::try_from(e)?);
                    } else {
                        ret.concatenate_child(Outline::try_from(e)?);
                    }
                }
                Ok(ret)
            }

            Titled { heading, contents } => {
                let mut ret = Outline::try_from(*heading)?;
                if ret.headline.is_none() || !ret.children.is_empty() {
                    Err("Bad heading")?;
                }
                for e in contents {
                    ret.concatenate_child(Outline::try_from(e)?);
                }
                Ok(ret)
            }
        }
    }
}

#[derive(Default)]
struct Serializer {
    heading: Option<Expr>,
    acc: Vec<Expr>,
}

impl Serializer {
    fn consume_acc(&mut self) -> Result<Expr> {
        let contents = std::mem::replace(&mut self.acc, Vec::new());

        if let Some(heading) = std::mem::replace(&mut self.heading, None) {
            Ok(Titled {
                heading: Box::new(heading),
                contents,
            })
        } else {
            Ok(List(contents))
        }
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Expr> { Ok(v.into()) }

    fn serialize_i8(self, v: i8) -> Result<Expr> { Ok(v.into()) }

    fn serialize_i16(self, v: i16) -> Result<Expr> { Ok(v.into()) }

    fn serialize_i32(self, v: i32) -> Result<Expr> { Ok(v.into()) }

    fn serialize_i64(self, v: i64) -> Result<Expr> { Ok(v.into()) }

    fn serialize_u8(self, v: u8) -> Result<Expr> { Ok(v.into()) }

    fn serialize_u16(self, v: u16) -> Result<Expr> { Ok(v.into()) }

    fn serialize_u32(self, v: u32) -> Result<Expr> { Ok(v.into()) }

    fn serialize_u64(self, v: u64) -> Result<Expr> { Ok(v.into()) }

    fn serialize_f32(self, v: f32) -> Result<Expr> { Ok(v.into()) }

    fn serialize_f64(self, v: f64) -> Result<Expr> { Ok(v.into()) }

    fn serialize_char(self, v: char) -> Result<Expr> {
        if v.is_whitespace() {
            return Err(Error::default());
        }

        Ok(v.into())
    }

    fn serialize_str(self, v: &str) -> Result<Expr> { Ok(v.into()) }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Expr> {
        unimplemented!();
    }

    fn serialize_none(self) -> Result<Expr> {
        // This is allowed for the intermediate Expr (for value in key-value pairs), but needs to
        // be scrubbed before converting to Outline.
        Ok(Atom(Nil))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Expr>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Expr> {
        unimplemented!();
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Expr> {
        unimplemented!();
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Expr> {
        unimplemented!();
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Expr>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Expr>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> { Ok(self) }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!();
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> { Ok(self) }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!();
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.acc.push(value.serialize(&mut Serializer::default())?);
        Ok(())
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.acc.push(value.serialize(&mut Serializer::default())?);
        Ok(())
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(&mut Serializer::default())?;
        if let Atom(key @ Word(_)) = key {
            // Value gets rewritten when we hit serialize_value.
            self.acc.push(Entry {
                key,
                value: Box::new(Atom(Nil)),
            });
            Ok(())
        } else {
            // Non-primitive non-word keys aren't supported.
            Err(Error::default())
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.acc.is_empty() {
            return Err(Error::default());
        }
        // Replace switcheroo for the key-value pair we wrote in serialize_key.
        let val = value.serialize(&mut Serializer::default())?;
        let idx = self.acc.len() - 1;
        if val.is_empty() {
            // Empty value, do not save.
            self.acc.pop();
        } else {
            if let Entry { ref mut value, .. } = self.acc[idx] {
                *value = Box::new(val);
            } else {
                return Err(Error::default());
            }
        }

        Ok(())
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(&mut Serializer::default())?;
        if !value.is_empty() {
            if key == MAGIC_HEADING_NAME {
                self.heading = Some(value);
            } else {
                self.acc.push(Entry {
                    key: Value::new(key),
                    value: Box::new(value),
                });
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = Expr;
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn end(self) -> Result<Expr> { self.consume_acc() }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Error(String);

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self { Error(msg.to_string()) }
}

impl error::Error for Error {
    fn description(&self) -> &str { &self.0 }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}
