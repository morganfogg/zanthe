use std::error::Error;
use std::io::{self, Stdout, Write};

use crossterm::{
    self,
    event::read,
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute},
};

use crate::ui::{Interface, TextStyle};

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
            text_style: TextStyle::new(),
        }
    }

    fn write(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
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
        self.stdout.flush()?;
        Ok(())
    }
}

impl Interface for EchoInterface {
    fn print(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        self.write(text)
    }

    fn print_char(&mut self, text: char) -> Result<(), Box<dyn Error>> {
        self.write(&text.to_string())
    }

    fn done(&mut self) -> Result<(), Box<dyn Error>> {
        queue!(self.stdout, ResetColor, SetAttribute(Attribute::Reset))?;
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
