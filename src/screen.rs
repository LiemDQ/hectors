use termion::color;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::input::TermRead;
use std::io::{stdout, stdin, Write};


const RESERVED_ROWS : u16 = 2 ; 


#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct ScreenSize {
    pub width: u16,
    pub height: u16
}

pub struct Screen {
    size: ScreenSize,
    _stdout: RawTerminal<std::io::Stdout>, //restore terminal state after exit
}

impl Screen {
    pub fn default() -> Result<Self, std::io::Error> {
        let (xsize, ysize) = termion::terminal_size()?;
        
        Ok(Self {
            size: ScreenSize { width: xsize, height: ysize.saturating_sub(RESERVED_ROWS) },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    pub fn clear() {
        print!("{}", termion::clear::All);
    }

    pub fn flush() -> Result <(), std::io::Error> {
        stdout().flush()
    }

    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine)
    }

    pub fn size(&self) -> &ScreenSize {
        &self.size
    }

    pub fn cursor_position(position: &Position){
        let Position {mut x, mut y} = position;
        x = x.saturating_add(1); //terminal cursor position is 1-indexed
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }



    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn set_bg_color(color: color::Rgb){
        print!("{}", color::Bg(color));
    }

    pub fn set_fg_color(color: color::Rgb){
        print!("{}", color::Fg(color));
    }

    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }

    pub fn invert_colors(){
        print!("{}", termion::style::Invert);
    }

    pub fn reset_all_formatting(){
        print!("{}", termion::style::Reset);
    }
}