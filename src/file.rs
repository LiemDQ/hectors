use std::fs;
use std::io::{Error, Write};
use std::fmt;
use std::path::Path;
use std::ffi::OsStr;

use crate::editor::SearchDirection;
use crate::row::Row;
use crate::screen::Position;

#[derive(Clone, Copy, Debug)]
pub enum FileType {
    C,
    Rust,
    Text
}

impl Default for FileType {
    fn default() -> Self {
        FileType::Text
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FileType {
    pub fn to_enum_str(&self) -> &'static str {
        match self {
            FileType::C => "C",
            FileType::Rust => "Rust",
            FileType::Text => "Text",
        }
    }
}

///Contains metadata used for syntax highlighting in a given file.
#[derive(Default)]
pub struct HighlightOptions {
    file_type: Option<FileType>,
    pub numbers: bool,
    pub strings: bool,
    pub characters: bool,
    pub comments: bool,
    pub multiline_comments: bool,
    keywords1: Vec<String>,
    keywords2: Vec<String>,
}

impl HighlightOptions {
    pub fn from(filename: &str) -> Self {
        
        Self {
            numbers: true,
            strings: true,
            file_type: Self::set_filetype(filename),
            ..Default::default()
        }
    }

    pub fn set_filetype(filename: &str) -> Option<FileType> {
        let extension = Path::new(&filename).extension().and_then(OsStr::to_str);

        match extension {
            Some("rs") => Some(FileType::Rust),
            Some("c") => Some(FileType::C),
            Some("txt") => Some(FileType::Text),
            Some(_) => None,
            None => None, 
        }

    }

    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.keywords1
    }

    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.keywords2
    }
}

pub struct File {
    rows: Vec<Row>,
    pub filename: Option<String>,
    pub dirty: bool,
    hl_opts: HighlightOptions,
}

impl File {
    pub fn open(filename: &str) -> Result<Self, std::io::Error>{
        let mut rows : Vec<Row> = Vec::new();
        let contents = fs::read_to_string(filename)?;
        for line in contents.lines() {
            rows.push(Row::from(line));
        }

        Ok(Self {
            rows: rows,
            filename: Some(String::from(filename)),
            dirty: false,
            hl_opts: HighlightOptions::from(filename)
        })
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn file_type(&self) -> Option<FileType> {
        self.hl_opts.file_type
    }
    
    pub fn default() -> Self {
        Self {
            rows: Vec::new(),
            filename: None,
            dirty: false,
            hl_opts: Default::default()
        }
    }

    pub fn save(&mut self) -> Result<usize, Error> {
        let mut nbytes: usize = 0;
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            self.hl_opts.file_type = HighlightOptions::set_filetype(filename);
            for row in &self.rows {
                nbytes += row.len();
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }

            self.dirty = false;
        }
        Ok(nbytes)
    }
    
    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }
    
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    
    pub fn mut_row(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }
    
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn unhighlight_rows(&mut self, start: usize){
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start){
            row.is_highlighted = false; 
        }
    }

    pub fn insert(&mut self, at: &Position, c: char){
        if at.y > self.rows.len(){
            return;
        }

        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }
        self.unhighlight_rows(at.y);
    }

    pub fn delete(&mut self, at: &Position){
        if at.y > self.rows.len(){
            return;
        }
        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < self.rows.len(){
            //do nothing for now, but the rows should be merged.
            let next_row = self.rows.remove(at.y+1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }

    fn insert_newline(&mut self, at: &Position){
        if at.y > self.rows.len() {
            return;
        }
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
        }
        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);

        self.rows.insert(at.y+1, new_row);
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = Position {x: at.x, y: at.y};

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };


        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(index) = row.find(query, position.x, direction){
                    position.x = index;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }

            } else {
                return None; 
            }
        }
        None
    }

    ///Highlights selected word in the text, and any highlighting options enabled.
    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>){
        let mut start_with_comment = false;
        let until = if let Some(until) = until {
            if until.saturating_add(1) < self.rows.len() {
                until.saturating_add(1)
            } else {
                self.rows.len()
            }
        } else {
            self.rows.len()
        };

        for row in &mut self.rows[..until] {
            start_with_comment = row.highlight(&self.hl_opts, word, start_with_comment);
        }
    }

}