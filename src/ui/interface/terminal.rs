use std::cell::RefCell;
use std::fs::File;
use std::io::{self, prelude::*, Stdout};
use std::mem::take;

use anyhow::{anyhow, Result};
use crossterm::{
    self,
    cursor::{position as cursor_pos, MoveLeft, MoveTo},
    event::{self, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use num_traits::FromPrimitive;
use tracing::{error, warn};

use crate::game::InputCode;
use crate::ui::interface::{ClearMode, Interface};
use crate::ui::Screen;
use crate::ui::TextStyle;

#[derive(Default)]
struct Buffer {
    pub elements: Vec<BufferElement>,
}

impl Buffer {
    fn put_text(&mut self, value: &str) {
            match self.elements.last_mut() {
                None | Some(BufferElement::Attribute(_)) => {
                    self.elements.push(BufferElement::Text(value.to_owned()));
                }
                Some(BufferElement::Text(s)) => {
                    s.push_str(value);
                }
            }
    }

    fn put_attribute(&mut self, attribute: Attribute) {
        self.elements.push(BufferElement::Attribute(attribute));
    }

    fn clear(&mut self) -> Vec<BufferElement> {
        take(&mut self.elements)
    }
}

enum BufferElement {
    Attribute(Attribute),
    Text(String),
}

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    text_style: TextStyle,
    active_screen: Screen,
    old_cursor_pos: (u16, u16),
    upper_window_height: u16,
    enable_buffering: bool,
    screen_style: TextStyle,
    transcript: File,
    z_machine_version: u8,
    buffer: Buffer,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(TerminalInterface {
            active_screen: Screen::Lower,
            text_style: TextStyle::default(),
            old_cursor_pos: (0, 0),
            upper_window_height: 0,
            enable_buffering: true,
            screen_style: TextStyle::default(),
            transcript: File::create("transcript.txt")?,
            z_machine_version: 5,
            buffer: Buffer::default(),
        })
    }

    /// Delete a character from the screen and the history.
    fn backspace(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        let (column, row) = cursor_pos()?;
        if column == 0 {
            let (width, _) = term_size()?;
            queue!(
                stdout,
                MoveTo(width - 1, row - 1),
                Print(" "),
                MoveTo(width - 1, row - 1),
            )?;
        } else {
            queue!(stdout, MoveLeft(1), Print(" "), MoveLeft(1),)?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn active_is_visible(&self) -> bool {
        match self.active_screen {
            Screen::Upper => self.upper_window_height != 0,
            Screen::Lower => self.upper_window_height <= term_size().unwrap().1,
        }
    }

    /// For printing to the upper screen where no buffering should take place
    fn print_unbufferable(&mut self, text: &str) -> Result<()> {
        self.flush_buffer()?;
        if self.active_is_visible() {
            let mut stdout = io::stdout();
            execute!(stdout, Print(text))?;
        }
        Ok(())
    }

    fn cursor_to_home(&self) -> Result<()> {
        if self.z_machine_version < 5 {
            todo!("DO THIS");
        } else {
            let mut stdout = io::stdout();
            queue!(stdout, MoveTo(0, self.upper_window_height))?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.elements.len() == 0 {
            return Ok(());
        }
        let mut stdout = io::stdout();
        for element in self.buffer.clear() {
            match element {
                BufferElement::Attribute(attr) => {
                    queue!(stdout, SetAttribute(attr))?;
                }
                BufferElement::Text(text) => {
                    queue!(stdout, Print(text.replace("\n", "\r\n")))?;
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }

    /// For printing to the lower screen where buffering and wrapping should be attempted.
    fn print_bufferable(&mut self, text: &str, immediate: bool) -> Result<()> {
        if immediate && self.active_is_visible() {
            let mut stdout = io::stdout();
            self.flush_buffer()?;
            execute!(stdout, Print(text.replace("\n", "\r\n")))?;
        } else {
            self.buffer.put_text(&text);
        }
        Ok(())
    }
}

impl Drop for TerminalInterface {
    /// Restore the terminal to its previous state when exiting.
    fn drop(&mut self) {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}

impl Interface for TerminalInterface {
    fn print(&mut self, text: &str) -> Result<()> {
        self.transcript.write_all(text.as_bytes())?;
        match self.active_screen {
            Screen::Lower => self.print_bufferable(text, !self.enable_buffering),
            Screen::Upper => self.print_unbufferable(text),
        }
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.print(&text.to_string())
    }

    fn buffer_mode(&mut self, enable_buffering: bool) -> Result<()> {
        self.enable_buffering = enable_buffering;
        Ok(())
    }

    fn get_screen_size(&self) -> (u16, u16) {
        return term_size().unwrap();
    }

    fn set_active(&mut self, split: u16) -> Result<()> {
        let new_active = Screen::from_u16(split).ok_or_else(|| anyhow!("Invalid screen"))?;

        let mut stdout = io::stdout();

        if self.active_screen == Screen::Lower {
            self.flush_buffer()?;
            self.old_cursor_pos = cursor_pos()?;
            error!("{:?}", self.old_cursor_pos);
        }

        if new_active == Screen::Upper {
            queue!(stdout, MoveTo(0, 0))?;
        } else {
            queue!(stdout, MoveTo(self.old_cursor_pos.0, self.old_cursor_pos.1))?;
        }
        stdout.flush()?;
        self.active_screen = new_active;
        Ok(())
    }

    fn split_screen(&mut self, split: u16) -> Result<()> {
        self.upper_window_height = split;
        Ok(())
    }

    // Set the location of the cursor
    fn set_cursor(&mut self, line: u16, column: u16) -> Result<()> {
        let mut stdout = io::stdout();
        if self.active_screen != Screen::Upper {
            warn!("Tried to call set_cursor outside upper window");
            return Ok(());
        }
        execute!(stdout, MoveTo(column - 1, line - 1))?;
        Ok(())
    }

    fn clear(&mut self, mode: ClearMode) -> Result<()> {
        let mut stdout = io::stdout();
        match mode {
            ClearMode::Full => {
                queue!(stdout, Clear(ClearType::All))?;
            }
            ClearMode::FullUnsplit => {
                self.split_screen(0)?;
                queue!(stdout, Clear(ClearType::All))?;
            }
            ClearMode::Single(v) => {
                panic!("AAAAAAAA");
            }
        }
        self.cursor_to_home()?;
        stdout.flush()?;
        Ok(())
    }

    fn read_char(&mut self) -> Result<InputCode> {
        self.flush_buffer()?;
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => return Ok(InputCode::Newline),
                    KeyCode::Char(c) => {
                        self.print_bufferable(&c.to_string(), true)?;
                        return Ok(InputCode::Character(c));
                    }
                    KeyCode::Up => return Ok(InputCode::CursorUp),
                    KeyCode::Down => return Ok(InputCode::CursorDown),
                    KeyCode::Left => return Ok(InputCode::CursorLeft),
                    KeyCode::Right => return Ok(InputCode::CursorRight),
                    KeyCode::Backspace | KeyCode::Delete => return Ok(InputCode::Delete),
                    KeyCode::Esc => return Ok(InputCode::Escape),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn read_line(&mut self, max_chars: usize) -> Result<String> {
        self.flush_buffer()?;
        let mut line = String::new();
        loop {
            match event::read()? {
                Event::Resize(..) => {
                    // Todo
                }
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => {
                        self.print_bufferable(&"\n", true)?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            self.print_bufferable(&c.to_string(), true)?;
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            self.backspace()?;
                            line.pop();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(line)
    }

    fn done(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        self.flush_buffer()?;
        queue!(stdout, Print("\n\r[Hit any key to exit...]"))?;
        stdout.flush()?;
        read()?;
        Ok(())
    }

    fn set_z_machine_version(&mut self, version: u8) {
        self.z_machine_version = version;
    }

    fn text_style_bold(&mut self) -> Result<()> {
        self.text_style.bold = true;
        if self.enable_buffering {
            self.buffer.put_attribute(Attribute::Bold);
        } else {
            queue!(io::stdout(), SetAttribute(Attribute::Bold))?;
        }
        Ok(())
    }

    fn text_style_emphasis(&mut self) -> Result<()> {
        self.text_style.emphasis = true;
        if self.enable_buffering {
            self.buffer.put_attribute(Attribute::Underlined);
        } else {
            queue!(io::stdout(), SetAttribute(Attribute::Underlined))?;
        }
        Ok(())
    }

    fn text_style_reverse(&mut self) -> Result<()> {
        self.text_style.reverse_video = true;
        if self.enable_buffering {
            self.buffer.put_attribute(Attribute::Reverse);
        } else {
            queue!(io::stdout(), SetAttribute(Attribute::Reverse))?;
        }
        Ok(())
    }

    fn text_style_fixed(&mut self) -> Result<()> {
        self.text_style.fixed_width = true;
        // TODO
        Ok(())
    }

    fn text_style_clear(&mut self) -> Result<()> {
        self.text_style = TextStyle::default();
        if self.enable_buffering {
            self.buffer.put_attribute(Attribute::Reset);
        } else {
            queue!(io::stdout(), SetAttribute(Attribute::Reset))?;
        }
        Ok(())
    }

    fn quit(&mut self) {}
}
