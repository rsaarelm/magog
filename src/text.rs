use std::vec::Vec;
use std::str;
use collections::Deque;
use collections::ringbuf::RingBuf;

pub struct WrapLineIterator<T> {
    /// Input iterator
    iter: T,
    /// Maximum line length
    line_len: uint,
    /// Characters output since last newline
    current_line_len: uint,
    /// Incoming elements
    buffer: RingBuf<char>,
    /// Peek window
    peek: Option<char>,
}

impl<T: Iterator<char>> WrapLineIterator<T> {
    fn fill_buffer(&mut self) {
        assert!(self.buffer.is_empty());
        let mut seen_text = false;
        let mut text_length = 0;
        match self.peek {
            Some(c) => {
                assert!(c.is_whitespace());
                self.buffer.push_back(c);
                self.peek = None;
            },
            None => ()
        }
        loop {
            let c = match self.iter.next() {
                None => break,
                Some(c) => c
            };

            // Break before non-newline whitespace after
            // non-whitespace text has been seen. (Newlines will be
            // included in the buffer.)
            if seen_text && c.is_whitespace() && c != '\n' {
                self.peek = Some(c);
                break;
            }
            self.buffer.push_back(c);
            if !c.is_whitespace() {
                seen_text = true;
                text_length += 1;
            }

            if text_length >= self.line_len {
                // The word is longer than the allowed line length,
                // need to break it.
                break;
            }
            // Break after hyphen or newline.
            if c == '-' || c == '\n' {
                break;
            }
        }
    }

    fn trim_buffer_left(&mut self) {
        while !self.buffer.is_empty() && self.buffer.front().unwrap().is_whitespace() {
            self.buffer.pop_front();
        }
    }
}

impl<T: Iterator<char>> Iterator<char> for WrapLineIterator<T> {
    fn next(&mut self) -> Option<char> {
        if self.buffer.is_empty() {
            return None;
        }
        if self.current_line_len + self.buffer.len() > self.line_len {
            // Next word won't fit, insert a newline.
            self.current_line_len = 0;
            self.trim_buffer_left();
            return Some('\n');
        }
        let c = self.buffer.pop_front().unwrap();
        self.current_line_len += 1;
        if c == '\n' {
            self.current_line_len = 0;
        }
        if self.buffer.is_empty() {
            self.fill_buffer();
        }
        Some(c)
    }
}

// All char iterators get this trait and the new method.
pub trait WrapUtil {
    fn wrap(self, line_len: uint) -> WrapLineIterator<Self>;
}

impl<T: Iterator<char>> WrapUtil for T {
    fn wrap(self, line_len: uint) -> WrapLineIterator<T> {
        assert!(line_len > 0);
        let mut ret = WrapLineIterator{
            iter: self,
            line_len: line_len,
            current_line_len: 0,
            buffer: RingBuf::new(),
            peek: None,
        };
        ret.fill_buffer();
        ret
    }
}

pub fn wrap_lines(line_len: uint, s: &str) -> ~str {
    str::from_chars(s.chars().wrap(line_len).collect::<Vec<char>>().as_slice())
}

/// Try to convert a char into the IBM PC code page 437 character set.
pub fn to_cp437(c: char) -> Option<u8> {
    static CP: &'static str =
       "\u0000☺☻♥♦♣♠•◘○◙♂♀♪♫☼\
        ►◄↕‼¶§▬↨↑↓→←∟↔▲▼ \
        !\"#$%&'()*+,-./\
        0123456789:;<=>?\
        @ABCDEFGHIJKLMNO\
        PQRSTUVWXYZ[\\]^_\
        `abcdefghijklmno\
        pqrstuvwxyz{|}~⌂\
        ÇüéâäàåçêëèïîìÄÅ\
        ÉæÆôöòûùÿÖÜ¢£¥₧ƒ\
        áíóúñÑªº¿⌐¬½¼¡«»\
        ░▒▓│┤╡╢╖╕╣║╗╝╜╛┐\
        └┴┬├─┼╞╟╚╔╩╦╠═╬╧\
        ╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀\
        αßΓπΣσµτΦΘΩδ∞φε∩\
        ≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u00a0";
    // XXX: O(n) search when we could use an O(1) look-up table.
    for (idx, enc) in CP.chars().enumerate() {
        if c == enc {
            return Some(idx as u8);
        }
    }
    None
}

pub struct Map2DIterator<T> {
    /// Input iterator
    iter: T,
    x: int,
    y: int,
}

impl<T: Iterator<char>> Iterator<(char, int, int)> for Map2DIterator<T> {
    fn next(&mut self) -> Option<(char, int, int)> {
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

pub trait Map2DUtil {
    fn map2d(self) -> Map2DIterator<Self>;
}

impl<T: Iterator<char>> Map2DUtil for T {
    fn map2d(self) -> Map2DIterator<T> {
        Map2DIterator{ iter: self, x: 0, y: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_small() {
        assert_eq!(wrap_lines(1, ""), "".to_owned());
        assert_eq!(wrap_lines(1, " "), " ".to_owned());
        assert_eq!(wrap_lines(1, "a"), "a".to_owned());
        assert_eq!(wrap_lines(4, "foo bar"), "foo\nbar".to_owned());
        assert_eq!(wrap_lines(8, "foo bar"), "foo bar".to_owned());
    }

    #[test]
    fn wrap_newlines() {
        // Newline in input means that the subsequent whitespace
        // needs to be preserved.
        let txt = "First\n  Second";
        assert_eq!(wrap_lines(8, txt), "First\n  Second".to_owned());
    }
}
