/*! String processing utilities */

/// Divide a string into the longest slice that fits within maximum line
/// length and the rest of the string.
///
/// Place the split before a whitespace or after a hyphen if possible. Any
/// whitespace between the two segments is trimmed. Newlines will cause a
/// segment split when encountered.
pub fn split_line<'a, F>(text: &'a str, char_width: &F, max_len: f32) -> (&'a str, &'a str)
    where F: Fn(char) -> f32 {
    assert!(max_len >= 0.0);

    if text.len() == 0 { return (text, text); }

    // Init the split position to 1 because we always want to return at least
    // 1 character in the head partition.
    let mut head_end = 1;
    let mut tail_start = 1;
    // Is the iteration currently consuming whitespace inside a possible
    // break.
    let mut eat_whitespace = false;
    let mut length = 0.0;

    for (i, c) in text.chars().enumerate() {
        length = length + char_width(c);

        // Invariant: head_end and tail_start describe a valid, but possibly
        // suboptimal return value at this point.
        assert!(text[..head_end].len() > 0);
        assert!(head_end <= tail_start);

        if eat_whitespace {
            if c.is_whitespace() {
                tail_start = i + 1;
                if c == '\n' { return (&text[..head_end], &text[tail_start..]); }
                continue;
            } else {
                eat_whitespace = false;
            }
        }

        // We're either just encountering the first whitespace after a block
        // of text, or over non-whitespace text.
        assert!(!eat_whitespace);

        // Invariant: The length of the string processed up to this point is
        // still short enough to return.

        // Encounter the first whitespace, set head_end marker.
        if c.is_whitespace() {
            head_end = i;
            tail_start = i + 1;
            if c == '\n' { return (&text[..head_end], &text[tail_start..]); }
            eat_whitespace = true;
            continue;
        }

        assert!(!c.is_whitespace());

        // Went over the allowed length.
        if length > max_len {
            if i > 1 && head_end == 1 && tail_start == 1 {
                // Didn't encounter any better cut points, so just place cut
                // in the middle of the word where we're at.
                head_end = i;
                tail_start = i;
            }

            // Use the last good result.
            return (&text[..head_end], &text[tail_start..]);
        }

        // Hyphens are a possible cut point.
        if c == '-' {
            head_end = i + 1;
            tail_start = i + 1;
        }
    }

    (&text, &""[..])
}

/// Wrap a text into multiple lines separated by newlines.
pub fn wrap_lines<F>(mut text: &str, char_width: &F, max_len: f32) -> String
    where F: Fn(char) -> f32 {
    let mut result = String::new();
    loop {
        let (head, tail) = split_line(text, char_width, max_len);
        if head.len() == 0 && tail.len() == 0 { break; }
        assert!(head.len() > 0, "Line splitter caught in infinite loop");
        assert!(tail.len() < text.len(), "Line splitter not shrinking string");
        result = result + head;
        // Must preserve a hard newline at the end if the input string had
        // one. The else branch checks for the very last char being a newline,
        // this would be clipped off otherwise.
        if tail.len() != 0 { result = result + "\n"; }
        else if text.chars().last() == Some('\n') { result = result + "\n"; }
        text = tail;
    }
    result
}

pub struct Map2DIterator<T> {
    /// Input iterator
    iter: T,
    x: i32,
    y: i32,
}

impl<T: Iterator<Item=char>> Iterator for Map2DIterator<T> {
    type Item = (char, i32, i32);

    fn next(&mut self) -> Option<(char, i32, i32)> {
        loop {
            match self.iter.next() {
                None => { return None }
                Some(c) if c == '\n' => { self.y += 1; self.x = 0; }
                Some(c) if (c as u32) < 32 => { }
                Some(c) => { self.x += 1; return Some((c, self.x - 1, self.y)) }
            }
        }
    }
}

pub trait Map2DUtil: Sized {
    /// Convert an input value into a sequence of 2D coordinates associated
    /// with a subvalue.
    ///
    /// Used for converting a string of ASCII art into characters and their
    /// coordinates.
    fn map2d(self) -> Map2DIterator<Self>;
}

impl<T: Iterator<Item=char>> Map2DUtil for T {
    fn map2d(self) -> Map2DIterator<T> {
        Map2DIterator{ iter: self, x: 0, y: 0 }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_split_line() {
        use super::split_line;

        assert_eq!(("", ""), split_line("", &|_| 1.0, 12.0));
        assert_eq!(("a", ""), split_line("a", &|_| 1.0, 5.0));
        assert_eq!(("the", "cat"), split_line("the cat", &|_| 1.0, 5.0));
        assert_eq!(("the", "cat"), split_line("the     cat", &|_| 1.0, 5.0));
        assert_eq!(("the", "cat"), split_line("the  \t cat", &|_| 1.0, 5.0));
        assert_eq!(("the", "cat"), split_line("the \ncat", &|_| 1.0, 32.0));
        assert_eq!(("the", "   cat"), split_line("the \n   cat", &|_| 1.0, 32.0));
        assert_eq!(("the  cat", ""), split_line("the  cat", &|_| 1.0, 32.0));
        assert_eq!(("the", "cat sat"), split_line("the cat sat", &|_| 1.0, 6.0));
        assert_eq!(("the cat", "sat"), split_line("the cat sat", &|_| 1.0, 7.0));
        assert_eq!(("a", "bc"), split_line("abc", &|_| 1.0, 0.01));
        assert_eq!(("dead", "beef"), split_line("deadbeef", &|_| 1.0, 4.0));
        assert_eq!(("the-", "cat"), split_line("the-cat", &|_| 1.0, 5.0));
    }
}
