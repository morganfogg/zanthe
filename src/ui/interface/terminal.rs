use std::fmt::Display;
use std::io::{self, Stdout, Write};

use anyhow::Result;
use crossterm::{
    self,
    cursor::{MoveLeft, MoveTo},
    event::{self, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::game::InputCode;
use crate::ui::interface::Interface;
use crate::ui::TextStyle;

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    stdout: Stdout,
    text_style: TextStyle,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(TerminalInterface {
            stdout,
            text_style: TextStyle::new(),
        })
    }

    fn write<T>(&mut self, text: T) -> Result<()>
    where
        T: Display + Clone,
    {
        if self.text_style.bold {
            queue!(self.stdout, SetAttribute(Attribute::Bold))?;
        }
        if self.text_style.emphasis {
            queue!(self.stdout, SetAttribute(Attribute::Underlined))?;
        }
        if self.text_style.reverse_video {
            queue!(self.stdout, SetAttribute(Attribute::Reverse))?;
        }
        queue!(
            self.stdout,
            Print(format!("{}", text).replace("\n", "\n\r")),
            SetAttribute(Attribute::Reset)
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

impl Drop for TerminalInterface {
    /// Restore the terminal to its previous state when exiting.
    fn drop(&mut self) {
        execute!(self.stdout, LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}

impl Interface for TerminalInterface {
    fn print(&mut self, text: &str) -> Result<()> {
        self.write(text)
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.print(&text.to_string())
    }

    fn clear(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn read_char(&mut self) -> Result<InputCode> {
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => return Ok(InputCode::Newline),
                    KeyCode::Char(c) => return Ok(InputCode::Character(c)),
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
        let mut line = String::new();
        loop {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Enter => {
                        self.write(&"\n")?;
                        break;
                    }
                    KeyCode::Esc => {
                        panic!("Yes");
                    }
                    KeyCode::Char(c) => {
                        if line.len() < max_chars {
                            self.write(c)?;
                            self.stdout.flush()?;
                            line.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if !line.is_empty() {
                            queue!(self.stdout, MoveLeft(1), Print(" "), MoveLeft(1))?;
                            self.stdout.flush()?;
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
        queue!(self.stdout, Print("\n\r[Hit any key to exit...]"))?;
        self.stdout.flush()?;
        read()?;
        Ok(())
    }

    fn text_style_bold(&mut self) {
        self.text_style.bold = true;
    }

    fn text_style_emphasis(&mut self) {
        self.text_style.emphasis = true;
    }

    fn text_style_reverse(&mut self) {
        self.text_style.reverse_video = true;
    }

    fn text_style_fixed(&mut self) {
        self.text_style.fixed_width = true;
    }

    fn text_style_clear(&mut self) {
        self.text_style.bold = false;
        self.text_style.emphasis = false;
        self.text_style.reverse_video = false;
        self.text_style.fixed_width = false;
    }

    fn quit(&mut self) {}
}
