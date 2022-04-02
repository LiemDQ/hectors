use std::time::{self, Duration};
use termion::event::Key;
use std::error;

use crate::row::Row;
use crate::file::File;
use crate::screen::{Screen, Position};

/* This is the main editor source file for hecto! 
This is a multiline comment to test the functionlity of syntax highlighting.
 */

const HECTO_VERSION: &str = env!("CARGO_PKG_VERSION");
const HECTOR_QUIT_TIMES: u8 = 3;
const MESSAGE_TIMEOUT: Duration = std::time::Duration::from_secs(5);

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward
}

struct StatusMessage {
    pub text: String,
    pub time: time::Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: time::Instant::now(),
            text: message
        }
    }
}

pub struct Editor {
    cursor: Position, //cursor Position
    offset: Position,
    scr: Screen,
    file: File,
    statusmsg: StatusMessage,
    quit_times: u8,
    should_quit: bool,
    highlighted_word: Option<String>,
}

fn die(e: &dyn error::Error) {
    Screen::clear();
    panic!("{}", e);
}


impl Editor {
    pub fn new(file: File) -> Result<Self, std::io::Error> {
        
        let screen = Screen::default()?;
        
        Ok(Self { 
            cursor: Default::default(), 
            offset: Default::default(),
            scr: screen, 
            file: file,
            statusmsg: StatusMessage { text: String::from("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = search"), time: time::Instant::now()},
            quit_times: 0,
            should_quit: false,
            highlighted_word: None,})
    }

    fn draw_rows(&self) {
        let height = self.scr.size().height;
        for terminal_row in 0..height {
            Screen::clear_current_line();
            if let Some(row) = self.file.row(
                self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.file.is_empty() && terminal_row == height /3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row){
        let width = self.scr.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_message_bar(&self) {
        Screen::clear_current_line();
        if time::Instant::now() - self.statusmsg.time <= MESSAGE_TIMEOUT {
            let width = self.scr.size().width as usize;
            let msg = if self.statusmsg.text.len() >= width /* single line multiline comment */ {
                &self.statusmsg.text[..width]
            } else {
                &self.statusmsg.text[..]
            };
            print!("{}\r", msg);

        } else {
            //if this is the last line printed, 
            //it should not generate a newline or else the screen printout will overflow
            print!("\r"); 
        }
    }

    /* Another multiline comment
       found here. 
     */
    fn draw_status_bar(&self) {
        let filename = if let Some(name) = &self.file.filename {
            name
        } else {
            "[No name]"
        };

        let modified = if self.file.dirty { "(modified)" } else { "" };
        
        let mut status_msg = format!(
            "{} - {} lines {}", 
            filename, self.file.num_rows(), modified);
            
            
            let right_msg = format!(
                "{} | {}/{} ",
                if let Some(ft) = self.file.file_type() {
                    ft.to_enum_str()
                } else {
                    "no ft"
                },
                self.cursor.y,
                self.file.num_rows(),
            );
            
            let width = self.scr.size().width as usize;
            let padding =  width - right_msg.len() - status_msg.len();
            let spaces = " ".repeat(padding.saturating_sub(1));
            
            status_msg = format!("{}{}{}", status_msg, spaces, right_msg);
            status_msg.truncate(width);
        Screen::invert_colors();
        println!("{}\r", status_msg);
        Screen::reset_all_formatting();
    }

    fn draw_welcome_message(&self){
        let mut welcome_msg = format!("Hecto editor -- version {}\r", HECTO_VERSION);
        let width = self.scr.size().width as usize;
        let len = welcome_msg.len();
        let padding = width.saturating_sub(len)/2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{}{}", spaces, welcome_msg);
        welcome_msg.truncate(width);
        println!("{}\r", welcome_msg);
    }

    fn save(&mut self){
        if self.file.filename.is_none() {
            let new_name = self.prompt(
                "Save as: ", 
                |_, _, _|{}).unwrap_or(None);
            if new_name.is_none() {
                self.statusmsg = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.file.filename = new_name;
        }

        if let Ok(n) = self.file.save() {
            self.statusmsg = StatusMessage::from(format!("{} bytes written to disk", n));
        } else {
            self.statusmsg = StatusMessage::from("Error writing to file.".to_string());
        }

    }

    fn search(&mut self){
        let saved_position = self.cursor.clone();
        let saved_offset = self.offset.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt("Search (Use ESC/Arrows/Enter): ", 
                |editor, key, query|{
                    let mut moved = false;
                    match key {
                        Key::Right | Key::Down => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        },
                        Key::Left | Key::Up => {
                            direction = SearchDirection::Backward;
                        },
                        _ => { direction = SearchDirection::Forward; }
                    };
                    if let Some(position) = 
                        editor.file.find(&query, &editor.cursor, direction) {
                            editor.cursor = position;
                            editor.scroll();
                    } else if moved {
                        editor.move_cursor(Key::Left);													
                    }
                    editor.highlighted_word = Some(query.to_string());
                }
            ).unwrap_or(None);
        
        if query.is_none() {
            self.cursor = saved_position;
            self.offset = saved_offset;
            self.scroll();
        }
        self.highlighted_word = None;
    }


    pub fn run(&mut self){
        loop {
            if let Err(e) = self.refresh_screen() {
                die(&e);
            }
            if self.should_quit {
                break;
            }
            if let Err(e) = self.process_keypress() {
                die(&e);
            }
            
        }
    }
    fn scroll(&mut self) {
        // self.cursor.x = 0; //no horizontal scrolling for now
        // if self.cursor.y < self.file.len() {
        //     self.cursor.x = 
        // }

        let height = self.scr.size().height as usize;
        let width = self.scr.size().width as usize;

        //update offsets based on cursor position.
        //if the offset if past the cursor position, scroll up so the cursor occupies the top line.
        if self.cursor.y < self.offset.y {
            self.offset.y = self.cursor.y;
        }

        if self.cursor.y >= self.offset.y + height {
            self.offset.y = self.cursor.y - height + 1;
        }

        if self.cursor.x < self.offset.x {
            self.offset.x = self.cursor.x;
        }
        if self.cursor.x >= self.offset.x + width {
            self.offset.x = self.cursor.x - width + 1;
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let key = Screen::read_key()?;

        match key {
            Key::Char(c) => {
                self.file.insert(&self.cursor, c);
                self.move_cursor(Key::Right);
                if c == '\n' {
                    let count = self.file.row(self.cursor.y).unwrap().get_prefix_len(" ");
                    for _ in 0..count {
                        self.move_cursor(Key::Right);
                    }
                }
            },
            Key::Ctrl('q') => {
                if self.file.dirty && self.quit_times > 0 {
                    //print warning message
                    self.statusmsg = StatusMessage::from(
                        format!("Warning! File has unsaved changes. Press Ctrl-Q {} more times to exit."
                        , self.quit_times));
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.should_quit = true;            
            },
            Key::Ctrl('s') => {
                self.save();
            },
            Key::Ctrl('f') => {
                self.search();
            },
            Key::Ctrl('h') => {},
            Key::Backspace => {
                if self.cursor.x > 0 || self.cursor.y > 0 {
                    self.move_cursor(Key::Left);
                    self.file.delete(&self.cursor);
                }
            },
            Key::Delete => {
                self.file.delete(&self.cursor)
            },
            Key::PageUp |
            Key::PageDown |
            Key::End |
            Key::Home |
            Key::Left |
            Key::Right |
            Key::Up |
            Key::Down => { self.move_cursor(key)},
            _ => {} //do nothing 
        }
        self.scroll();            
        self.quit_times = HECTOR_QUIT_TIMES;
        Ok(())
    }

    fn move_cursor(&mut self, key: Key){
        let Position { mut x, mut y} = self.cursor;
        let height = self.file.len();
        let terminal_height = self.scr.size().height as usize;
        let width = if let Some(row) = self.file.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            //TODO: handle errors properly, and avoid panicking
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.file.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            },
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    x = 0;
                    y += 1;
                }
            }
            Key::Up => { 
                y = y.saturating_sub(1);
                if let Some(row) = self.file.row(y) {
                    if x > row.len() {
                        x = row.len()
                    }
                }
            },
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                    if let Some(row) = self.file.row(y) {
                        if x > row.len() {
                            x = row.len()
                        }
                    }
                }
            },
            Key::Home => x = 0,
            Key::End => x = width,
            Key::PageDown => y = if y.saturating_add(terminal_height) < height {
                y.saturating_add(terminal_height)
            } else {
                height
            },
            Key::PageUp => y = if y > terminal_height {
                y.saturating_sub(terminal_height)
            } else {
                0
            },
            _ => {}
        }

        self.cursor = Position {x, y}
    }

    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Screen::cursor_hide();
        Screen::cursor_position(&Position::default());
        if self.should_quit {
            Screen::clear();
        } else {
            self.file.highlight(&self.highlighted_word, 
                Some(self.offset.y.saturating_add(self.scr.size().height as usize)));
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Screen::cursor_position(&Position {
                x: self.cursor.x.saturating_sub(self.offset.x),
                y: self.cursor.y.saturating_sub(self.offset.y),
            });
        }
        Screen::cursor_show();
        Screen::flush()
    }

    fn prompt<Cb>(&mut self, prompt: &str, mut callback: Cb) -> Result<Option<String>, std::io::Error>
        where Cb: FnMut(&mut Self, Key, &String),
    {
        let mut msg = String::new();
        loop {
            self.statusmsg = StatusMessage::from(format!("{}{}*", prompt, msg));
            self.refresh_screen()?;
            let key = Screen::read_key()?;
            match key {
                Key::Backspace => msg.truncate(msg.len().saturating_sub(1)),
                Key::Char('\n') => {
                    break;
                }
                Key::Char(c) => {
                    if !c.is_control(){
                        msg.push(c);
                    }
                }
                Key::Esc => {
                    msg.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &msg);
        }
        self.statusmsg = StatusMessage::from(String::new());
        if msg.is_empty() {
            return Ok(None);
        }
        Ok(Some(msg))
    }
}
