use std::fmt::Display;
use std::io::{self, Stdout, Write};

use anyhow::Result;
use crossterm::{
    self,
    cursor::MoveLeft,
    event::read,
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute},
    terminal::{Clear, ClearType},
};

use crate::game::InputCode;
use crate::ui::interface::Interface;
use crate::ui::TextStyle;

/// A less advanced terminal interface that just echos instead of using a TUI.
pub struct EchoInterface {
    stdout: Stdout,
    text_style: TextStyle,
}

impl EchoInterface {
    pub fn new() -> EchoInterface {
        let stdout = io::stdout();
        EchoInterface {
            stdout,
            text_style: TextStyle::default(),
        }
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
        queue!(self.stdout, Print(text), SetAttribute(Attribute::Reset))?;
        Ok(())
    }
}

impl Interface for EchoInterface {
    fn print(&mut self, text: &str) -> Result<()> {
        self.write(&text)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn print_char(&mut self, text: char) -> Result<()> {
        self.write(&text.to_string())?;
        self.stdout.flush()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn done(&mut self) -> Result<()> {
        queue!(self.stdout, ResetColor, SetAttribute(Attribute::Reset))?;
        self.stdout.flush()?;
        read()?;
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
        while let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Enter => {
                    self.write(&"\n")?;
                    self.stdout.flush()?;
                    break;
                }
                KeyCode::Char(c) => {
                    if line.len() < max_chars {
                        self.write(&c)?;
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
            }
        }
        Ok(line)
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
