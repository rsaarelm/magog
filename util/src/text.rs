use std::vec::Vec;
use collections::ring_buf::RingBuf;

pub struct WrapLineIterator<T> {
    /// Input iterator
    iter: T,
    /// Maximum line length
    line_len: u32,
    /// Characters output since last newline
    current_line_len: u32,
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
    fn wrap(self, line_len: u32) -> WrapLineIterator<Self>;
}

impl<T: Iterator<Item=char>> WrapUtil for T {
    fn wrap(self, line_len: u32) -> WrapLineIterator<T> {
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

pub fn wrap_lines(line_len: u32, s: &str) -> String {
    s.chars().wrap(line_len).collect::<Vec<char>>().as_slice().collect()
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

pub trait Map2DUtil {
    fn map2d(self) -> Map2DIterator<Self>;
}

impl<T: Iterator<Item=char>> Map2DUtil for T {
    fn map2d(self) -> Map2DIterator<T> {
        Map2DIterator{ iter: self, x: 0, y: 0 }
    }
}
