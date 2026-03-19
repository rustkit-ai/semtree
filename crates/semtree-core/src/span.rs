use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

impl Span {
    pub fn new(start_byte: usize, end_byte: usize, start_line: usize, end_line: usize) -> Self {
        Self {
            start_byte,
            end_byte,
            start_line,
            end_line,
        }
    }

    pub fn byte_len(&self) -> usize {
        self.end_byte - self.start_byte
    }

    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }
}
