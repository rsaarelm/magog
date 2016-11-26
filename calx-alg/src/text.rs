//! String processing utilities

/// Split a long line into multiple lines that fit a given width.
///
/// Will treat newlines in the input as regular whitespace, you probably want to split your input
/// at newlines before using `split_line` on the individual lines.
pub fn split_line<'a, F>(text: &'a str, char_width: F, max_width: f32) -> LineSplit<'a, F>
    where F: Fn(char) -> f32
{
    LineSplit {
        remain: text,
        char_width: char_width,
        max_width: max_width,
        finished: false,
    }
}

pub struct LineSplit<'a, F> {
    remain: &'a str,
    char_width: F,
    max_width: f32,
    finished: bool,
}

impl<'a, F> Iterator for LineSplit<'a, F>
    where F: Fn(char) -> f32
{
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.finished {
            return None;
        }

        struct State {
            total_width: f32,
            clip_pos: usize,
            last_word_break: Option<(usize, f32)>,
            prev: char,
        }

        impl State {
            fn new() -> State {
                State {
                    total_width: 0.0,
                    clip_pos: 0,
                    last_word_break: None,
                    prev: 'A',
                }
            }

            fn update<F: Fn(char) -> f32>(
                &mut self,
                char_width: &F,
                c: char
            ) -> Option<(usize, f32)> {
                if c.is_whitespace() && !self.prev.is_whitespace() {
                    self.last_word_break = Some((self.clip_pos, self.total_width));
                }
                self.clip_pos += c.len_utf8();
                self.total_width += char_width(c);
                self.prev = c;

                // Return the cut in the current word if there is no last_word_break set yet.
                Some(self.last_word_break.unwrap_or((self.clip_pos, self.total_width)))
            }
        }

        let end_pos = {
            self.remain
                .chars()
                .chain(Some(' ')) // Makes the ending of the last word in line show up.
                .scan(State::new(), |s, c| s.update(&self.char_width, c))
                .scan(true, |is_first, (i, w)| {
                    // Always return at least one element.
                    // Past that return the last element that fits in the space.
                    if *is_first {
                        *is_first = false;
                        Some(i)
                    } else {
                        if w <= self.max_width { Some(i) } else { None }
                    }
                })
                .last()
                .unwrap_or(0)
        };

        let ret = &self.remain[..end_pos];

        self.remain = &self.remain[end_pos..];
        // Strip whitespace between this line and the next.
        let start_pos = self.remain
                            .chars()
                            .take_while(|&c| c.is_whitespace())
                            .map(|c| c.len_utf8())
                            .sum();
        self.remain = &self.remain[start_pos..];
        if self.remain.is_empty() {
            self.finished = true;
        }

        Some(ret)
    }
}


pub struct Map2DIterator<T> {
    /// Input iterator
    iter: T,
    x: i32,
    y: i32,
}

impl<T: Iterator<Item = char>> Iterator for Map2DIterator<T> {
    type Item = (char, i32, i32);

    fn next(&mut self) -> Option<(char, i32, i32)> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(c) if c == '\n' => {
                    self.y += 1;
                    self.x = 0;
                }
                Some(c) if (c as u32) < 32 => {}
                Some(c) => {
                    self.x += 1;
                    return Some((c, self.x - 1, self.y));
                }
            }
        }
    }
}

pub trait Map2DUtil: Sized {
    /// Convert an input value into a sequence of 2D coordinates associated with a subvalue.
    ///
    /// Used for converting a string of ASCII art into characters and their
    /// coordinates.
    fn map2d(self) -> Map2DIterator<Self>;
}

impl<T: Iterator<Item = char>> Map2DUtil for T {
    fn map2d(self) -> Map2DIterator<T> {
        Map2DIterator {
            iter: self,
            x: 0,
            y: 0,
        }
    }
}

/// Parser method generator.
///
/// Invoke inside an impl block. Declare a list of signatures of methods callable on `&mut self`.
/// The macro constructs a `parse(&mut self, input: &str) -> Result<(), String>` method that parses
/// input strings like "method_name 4 foo" into calls of `self.method_name(FromStr::from_str("4")?,
/// FromStr::from_str("foo")?)`.
///
///     # #[macro_use] extern crate calx_alg;
///     # fn main() {
///
///     struct Adder {
///         pub num: u32,
///     }
///
///     impl Adder {
///         fn add(&mut self, amount: u32) {
///             self.num += amount;
///         }
///
///         command_parser!{
///             fn add(&mut self, amount: u32);
///         }
///     }
///
///     let mut a = Adder { num: 1 };
///     a.add(2);
///     assert_eq!(a.num, 3);
///
///     assert!(a.parse("foobie bletch").is_err());
///     assert!(a.parse("add").is_err());       // Too few arguments
///     assert!(a.parse("add derp").is_err());  // Arguments won't parse to unsigned integer.
///     assert!(a.parse("add -1").is_err());
///     assert!(a.parse("add 1.0").is_err());
///
///     // NB: Due to limits of the macro implementation, trailing arguments
///     // are ignored instead of causing an error.
///     // assert!(a.parse("add 1 2").is_err());   // Too many arguments
///
///     assert_eq!(a.num, 3);
///
///     assert!(a.parse("add 3").is_ok());
///     assert_eq!(a.num, 6);
///
///     # }
#[macro_export]
macro_rules! command_parser {
    {
        $(fn $method:ident(&mut self$(, $argname:ident: $argtype:ty)*);)+
    } => {
        fn parse(&mut self, input: &str) -> Result<(), Box<::std::error::Error>> {
            #[derive(Debug)]
            struct ParseError(String);

            impl ::std::fmt::Display for ParseError {
                fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                    self.0.fmt(fmt)
                }
            }

            impl ::std::error::Error for ParseError {
                fn description(&self) -> &str { &self.0 }
            }

            fn ParseErr(s: String) -> Result<(), Box<::std::error::Error>> {
                Err(Box::new(ParseError(s)))
            }

            use ::std::str::FromStr;
            let mut elts = input.split(" ");
            let cmd = elts.next();
            if let Some(cmd) = cmd {
                match cmd {
                    $(
                    stringify!($method) => {
                        self.$method(
                            $({
                                if let Some(arg) = elts.next() {
                                    let param: $argtype = FromStr::from_str(arg)?;
                                    param
                                } else {
                                    return ParseErr(format!("Too few arguments"));
                                }
                            }),*
                        );
                        Ok(())
                    }
                    )+
                    x => ParseErr(format!("Unknown command {}", x)),
                }
            } else {
                return ParseErr(format!("No input"));
            }
        }
    }
}
