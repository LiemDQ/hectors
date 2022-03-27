use unicode_segmentation::UnicodeSegmentation;
use crate::{highlight::Highlight, editor::SearchDirection, file::HighlightOptions};
use std::{cmp, thread::current};
use termion::color;

const HECTO_TAB_STOP: usize = 4;
const HECTO_TAB_SPACE: &str = " ";
#[derive(Default)]
pub struct Row {
    pub string: String,
    highlight: Vec<Highlight>,
    pub is_highlighted: bool,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlight: Vec::new(),
            is_highlighted: false,
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let start = cmp::min(start, end);
        let end = cmp::min(end, self.string.len());
        let mut result = String::new();
        let mut current_highlighting = &Highlight::None;
        for (index,grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end-start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self.highlight
                    .get(index)
                    .unwrap_or(&Highlight::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight =
                        format!("{}", termion::color::Fg(highlighting_type.to_true_color()));
                    result.push_str(&start_highlight[..]);
                } 
                
                if grapheme == "\t" {
                    result.push_str(HECTO_TAB_SPACE);
                } else {
                    result.push_str(grapheme);
                }
            }
        }
        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }

    pub fn insert(&mut self, at: usize, c: char){
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate(){
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }

        self.len = length;
        self.string = result;
    }

    pub fn delete(&mut self, at: usize){
        if at >= self.len() {
            return;
        }
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate(){
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }

        self.len = length;
        self.string = result;
    }

    pub fn append(&mut self, new: &Self){
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
            return None;
        }
        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };

        //find the associated byte index matching the query, if any.
        let substr: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end-start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substr.find(query)
        } else {
            substr.rfind(query)
        };

        //the grapheme index is the number of spaces the cursor has to move
        //while the byte index is the actual displacement in the byte array
        //for moving the cursor position as a result of the search operation
        //we need the grapheme index, which can be obtained from an enumerate iterator.
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substr.grapheme_indices(true).enumerate() 
            {
                if matching_byte_index == byte_index {
                    return Some(start + grapheme_index);
                }
            }
        }
        
        None
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut new_row: String = String::new();
        let mut new_length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                new_row.push_str(grapheme);
                new_length += 1;
            }
        }
        self.string = row;
        self.len = length;
        self.is_highlighted = false;
        Self {
            string: new_row,
            len: new_length,
            is_highlighted: false,
            highlight: Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn highlight(&mut self, hl_opts: &HighlightOptions, word: &Option<String>, start_with_comment: bool) -> bool {
        let chars: Vec<char> = self.string.chars().collect();
        for c in chars {
            if c.is_numeric() {
                self.highlight.push(Highlight::Number);
            } else {
                self.highlight.push(Highlight::Normal);
            }
        }

        true
    }

}