use serde::{de, Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// A string-like type that's guaranteed to be a single word without whitespace.
///
/// Used in outline data declarations, inline lists must consist of symbol-like values.
#[derive(Clone, Eq, PartialEq, Serialize, Debug)]
pub struct Sym<T: AsRef<str>>(T);

impl<T: AsRef<str>> std::ops::Deref for Sym<T> {
    type Target = str;

    fn deref(&self) -> &Self::Target { self.0.as_ref() }
}

impl<T: AsRef<str>> Sym<T> {
    pub fn new<U: Into<T>>(value: U) -> Result<Self, ()> {
        let value = value.into();

        if value.as_ref().is_empty() {
            return Err(());
        }
        if value.as_ref().chars().any(|c| c.is_whitespace()) {
            return Err(());
        }
        Ok(Sym(value))
    }
}

impl<'a, T: AsRef<str> + FromStr<Err = E>, E: Error + 'static> FromStr for Sym<T> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = T::from_str(s)?;
        match Sym::new(inner) {
            Err(_) => {
                return Err("err")?;
            }
            Ok(ok) => {
                return Ok(ok);
            }
        }
    }
}

impl<T: AsRef<str> + Ord> Ord for Sym<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Special case "*" to always sort last so that tag lists containing a "*" tag get it at
        // the end of the list and show up as important lines in outline syntax highlighting
        match (&self.0, &other.0) {
            (x, y) if x == y => Ordering::Equal,
            (_, y) if y.as_ref() == "*" => Ordering::Less,
            (x, _) if x.as_ref() == "*" => Ordering::Greater,
            (x, y) => x.cmp(y),
        }
    }
}

impl<T: AsRef<str> + Ord> PartialOrd for Sym<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl<T: AsRef<str>> fmt::Display for Sym<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0.as_ref()) }
}

impl<'de, T: AsRef<str> + Deserialize<'de> + fmt::Debug> Deserialize<'de> for Sym<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = T::deserialize(deserializer)?;
        match Sym::new(inner) {
            Ok(ret) => Ok(ret),
            Err(_) => Err(de::Error::custom("Invalid symbol")),
        }
    }
}

#[macro_export]
macro_rules! sym {
    ($fmt:expr) => {
        $crate::Sym::new($fmt).expect("Invalid symbol")
    };

    ($fmt:expr, $($arg:expr),*) => {
        $crate::Sym::new(format!($fmt, $($arg),*)).expect("Invalid symbol")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol() {
        type Symbol<'a> = Sym<&'a str>;

        assert!(Symbol::new("foobar").is_ok());
        assert!(Symbol::new("").is_err());
        assert!(Symbol::new("foo bar").is_err());
        assert!(Symbol::new("  foobar").is_err());
        assert!(Symbol::new("foo\nbar").is_err());
        assert!(Symbol::new("foobar\n").is_err());

        let mut tags: Vec<Symbol> = vec![sym!("b"), sym!("*"), sym!("a")];
        tags.sort();
        let tags: Vec<String> = tags.into_iter().map(|c| c.to_string()).collect();
        assert_eq!(tags.join(" "), "a b *");
    }

    #[test]
    fn test_symbol_literal() {
        let s1: Sym<String> = sym!("foo");
        let s2: Sym<&str> = sym!("bar");
        assert_eq!(&format!("{}", s1), "foo");
        assert_eq!(&format!("{}", s2), "bar");
    }
}
