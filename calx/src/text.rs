//! String processing utilities

use std::ops::{Add, AddAssign};

/// Split a long line into multiple lines that fit a given width.
///
/// Will treat newlines in the input as regular whitespace, you probably want to split your input
/// at newlines before using `split_line` on the individual lines.
pub fn split_line<F, N>(text: &str, char_width: F, max_width: N) -> LineSplit<'_, F, N>
where
    F: Fn(char) -> N,
    N: Add + PartialOrd + Copy,
{
    LineSplit {
        remain: text,
        char_width,
        max_width,
        finished: false,
    }
}

pub struct LineSplit<'a, F, N> {
    remain: &'a str,
    char_width: F,
    max_width: N,
    finished: bool,
}

impl<'a, F, N> Iterator for LineSplit<'a, F, N>
where
    F: Fn(char) -> N,
    N: Default + Add + AddAssign + Copy + PartialOrd,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.finished {
            return None;
        }

        struct State<N> {
            total_width: N,
            clip_pos: usize,
            last_word_break: Option<(usize, N)>,
            prev: char,
        }

        impl<N: Default + Add + AddAssign + Copy> State<N> {
            fn new() -> State<N> {
                State {
                    total_width: N::default(),
                    clip_pos: 0,
                    last_word_break: None,
                    prev: 'A',
                }
            }

            fn update<F: Fn(char) -> N>(&mut self, char_width: &F, c: char) -> Option<(usize, N)> {
                if c.is_whitespace() && !self.prev.is_whitespace() {
                    self.last_word_break = Some((self.clip_pos, self.total_width));
                }
                self.clip_pos += c.len_utf8();
                self.total_width += char_width(c);
                self.prev = c;

                // Return the cut in the current word if there is no last_word_break set yet.
                Some(
                    self.last_word_break
                        .unwrap_or((self.clip_pos, self.total_width)),
                )
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
                    } else if w <= self.max_width {
                        Some(i)
                    } else {
                        None
                    }
                })
                .last()
                .unwrap_or(0)
        };

        let ret = &self.remain[..end_pos];

        self.remain = &self.remain[end_pos..];
        // Strip whitespace between this line and the next.
        let start_pos = self
            .remain
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

/// Translate segments in square brackets in string with the given function.
///
/// Square brackets can be escaped by doubling them, `[[` becomes a literal `[` and `]]` becomes a
/// literal `]`.
///
/// If your opening and closing brackets don't match, the formatting behavior is unspecified.
///
/// # Examples
///
/// ```
/// use calx::templatize;
///
/// fn translate(word: &str) -> Result<String, ()> {
///     match word {
///         "foo" => Ok("bar"),
///         _ => Err(())
///     }.map(|x| x.to_string())
/// }
///
/// assert_eq!(Ok("foo bar baz".to_string()), templatize(translate, "foo [foo] baz"));
/// assert_eq!(Err(()), templatize(translate, "foo [bar] baz"));
/// assert_eq!(Ok("foo [foo] baz".to_string()), templatize(translate, "foo [[foo]] baz"));
/// ```
pub fn templatize<F, E>(mut mapper: F, mut text: &str) -> Result<String, E>
where
    F: FnMut(&str) -> Result<String, E>,
{
    // I'm going to do some fun corner-cutting here.
    //
    // Instead of being all proper-like with the opening and closing bracket, I'll just treat them
    // both as a generic separator char, so the string will start in verbatim mode and a lone
    // bracket in either direction will toggle modes between verbatim and templatize.

    fn next_chunk(text: &str) -> (String, &str) {
        let mut acc = String::new();
        let mut prev = '\0';
        for (i, c) in text.char_indices() {
            // Escaped bracket, emit one.
            if (c == '[' || c == ']') && prev == c {
                acc.push(c);
                prev = '\0';
                continue;
            }
            // Actual bracket, end chunk here and return remain.
            if prev == '[' || prev == ']' {
                return (acc, &text[i..]);
            }
            if c != '[' && c != ']' {
                acc.push(c);
            }
            prev = c;
        }
        (acc, &text[text.len()..])
    }

    let mut ret = String::new();
    let mut templating = false;
    while !text.is_empty() {
        let (mut chunk, remain) = next_chunk(text);
        text = remain;

        if templating {
            chunk = mapper(&chunk)?;
        }

        ret += &chunk;
        templating = !templating;
    }
    Ok(ret)
}
