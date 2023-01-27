pub mod window;

use std::fs::File;
use std::io::{self, prelude::*};
use std::mem::take;

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

use crate::game::Result;
use crate::interface::{Interface, ClearMode, InputCode};
use super::window::{Window, WindowManager, SplitDirection};

pub struct TerminalInterface {
    window_manager: WindowManager,
    upper_window_id: usize,
    lower_window_id: usize,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface> {
        Ok(TerminalInterface {
            upper_window_id: 0,
            lower_window_id: 0,
            window_manager: WindowManager::default(),
        })
    }
}

impl Interface for TerminalInterface {
    fn init(&mut self) -> Result<()> {
        WindowManager::init()?;
        self.lower_window_id = self.window_manager.split(0, SplitDirection::Above, 0)?;
        self.upper_window_id = self.window_manager.split(0, SplitDirection::Above, 0)?;
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<()> {
        self.window_manager.print(text)?;
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.window_manager.print_char(text)?;
    }

    fn done(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        print!("[Press any key to exit]");
        stdout.flush()?;
        Ok(())
    }

    fn clear(&mut self, mode: ClearMode) -> Result<()> {
        todo!();
    }

    fn text_style_bold(&mut self) -> Result<()> {
        self.window_manager.bold()?;
        Ok(())
    }

    fn text_style_emphasis(&mut self) -> Result<()> {
        self.window_manager.emphasis()?;
        Ok(())
    }

    fn text_style_reverse(&mut self) -> Result<()> {
        self.window_manager.reverse()?;
        Ok(())
    }

    fn text_style_fixed(&mut self) -> Result<()> {
        Ok(())
    }

    fn text_style_clear(&mut self) -> Result<()> {
        self.window_manager.reset_style()?;
        Ok(())
    }

    fn set_z_machine_version(&mut self, version: u8) {
        todo!();
    }

    fn read_char(&mut self) -> Result<InputCode> {
        self.window_manager.flush()?;
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
                        self.print_char('\n', true)?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            self.print_bufferable(&c.to_string(), true)?;
                            self.window_manager.flush()
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            self.backspace()?;
                            self.window_manager.flush()
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
        todo!();
    }

    fn get_screen_size(&self) -> (u16, u16) {
        term_size()
    }

    fn set_active(&mut self, active: u16) -> Result<()> {
        todo!();
    }

    fn set_cursor(&mut self, line: u16, column: u16) -> Result<()> {
        todo!();
    }

    fn buffer_mode(&mut self, enable: bool) -> Result<()> {
        todo!();
    }

    fn quit(&mut self) {
        todo!();
    }
}
