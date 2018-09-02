use track::Posn;
use std::collections::VecDeque;

pub struct Reader {
    chars: Vec<char>,
    ahead: VecDeque<char>,
    curr_index: usize,
    curr_posn: Posn

}

impl Reader {
    pub fn new<I>(chars: I) -> Reader where I: Iterator<Item=char> {
        Reader {
            chars: chars.collect(),
            ahead: VecDeque::with_capacity(4),
            curr_index: 0,
            curr_posn: Posn::origin()
        }
    }

    pub fn peek(&mut self, n: usize) -> Option<char> {
        debug_assert!(n < self.ahead.capacity(), "Lookahead buffer can't hold that many items");
        for _ in self.ahead.len()..(n + 1) {
            match self.chars.get(self.curr_index) {
                Some(ch) => {
                    self.ahead.push_back(*ch)
                }
                None => {
                    return None
                }
            }
        }
        self.ahead.get(n).map(|&x| x)
    }

    pub fn curr_posn(&self) -> Posn { self.curr_posn }

    pub fn curr_index(&self) -> usize { self.curr_index }

    pub fn seek(&mut self, last_index: usize, last_posn: Posn) {
        self.curr_index = last_index;
        self.curr_posn = last_posn;
    }
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let curr_char = self.ahead.pop_front().or_else(|| {
            self.chars.get(self.curr_index).map(|x| *x)
        });
        self.curr_index += 1;

        if (curr_char == Some('\r') && self.peek(0) != Some('\n')) ||
           curr_char == Some('\n') ||
           curr_char == Some('\u{2028}') ||
           curr_char == Some('\u{2029}') {
            self.curr_posn.line += 1;
            self.curr_posn.column = 0;
        } else {
            self.curr_posn.column += 1;
        }

        self.curr_posn.offset += 1;

        curr_char
    }
}
