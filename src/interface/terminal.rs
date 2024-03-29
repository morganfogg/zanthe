pub mod window;

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

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
use tracing::warn;

use crate::game::Result;
use crate::interface::{ClearMode, InputCode, Interface};
use window::{Constraint, Direction, TextStream, WindowKind, WindowManager};

pub struct TerminalInterface {
    wm: WindowManager,
    upper_screen_id: usize,
    lower_screen_id: usize,
}

impl TerminalInterface {
    pub fn new() -> Result<Self> {
        Ok(Self {
            wm: WindowManager::new(),
            upper_screen_id: 0,
            lower_screen_id: 0,
        })
    }
}

impl Interface for TerminalInterface {
    fn init(&mut self) -> Result<()> {
        self.wm.init()?;
        self.lower_screen_id = self.wm.split(
            0,
            Direction::Above,
            Constraint::RightFixed(0),
            WindowKind::TextStream(TextStream::default()),
        )?;
        self.upper_screen_id = self.wm.split(
            self.lower_screen_id,
            Direction::Above,
            Constraint::RightFixed(5),
            WindowKind::TextStream(TextStream::default()),
        )?;
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<()> {
        self.wm.print(text, false);
        Ok(())
    }

    /// Print a single character to the UI
    fn print_char(&mut self, text: char) -> Result<()> {
        self.wm.print_char(text, false);
        Ok(())
    }

    /// Clear the entire window
    fn clear(&mut self, mode: ClearMode) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// The game exited successfully, show a message then quit
    fn done(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Set the text style to bold
    fn text_style_bold(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Set the text style to emphais (italics)
    fn text_style_emphasis(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Set the text style to reverse video.
    fn text_style_reverse(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Set the text style to fixed-width
    fn text_style_fixed(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Remove all text styles
    fn text_style_clear(&mut self) -> Result<()> {
        // todo!();
        Ok(())
    }

    fn set_z_machine_version(&mut self, version: u8) {
        // todo!();
    }

    fn read_char(&mut self) -> Result<InputCode> {
        self.wm.flush_buffer();
        self.wm.set_active(self.lower_screen_id)?;
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => return Ok(InputCode::Newline),
                    KeyCode::Char(c) => {
                        self.wm.print_char(c, true)?;
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
        self.wm.flush_buffer();
        self.wm.set_active(self.lower_screen_id)?;
        let mut line = String::new();
        loop {
            match event::read()? {
                Event::Resize(..) => {
                    // Todo
                }
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => {
                        self.wm.print_char('\n', true)?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            self.wm.print_char(c, true)?;
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            self.wm.backspace();
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

    fn split_screen(&mut self, split: u16) -> Result<()> {
        // todo!();
        Ok(())
    }

    fn get_screen_size(&self) -> (u16, u16) {
        self.wm.size()
    }

    fn set_active(&mut self, active: u16) -> Result<()> {
        self.wm.set_active(match active {
            1 => self.upper_screen_id,
            0 => self.lower_screen_id,
            _ => todo!(),
        });
        Ok(())
    }

    fn set_cursor(&mut self, line: u16, column: u16) -> Result<()> {
        // todo!();
        Ok(())
    }

    fn buffer_mode(&mut self, enable: bool) -> Result<()> {
        // todo!();
        Ok(())
    }

    /// Close the UI immediately.
    fn quit(&mut self) {
        todo!();
    }
}
