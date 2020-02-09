use std::error::Error;
use std::io::{self, Stdout, Write};

use crossterm::{
    self,
    cursor::MoveTo,
    event::read,
    execute, queue,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use log::info;
use textwrap::fill;

use crate::ui::{Interface, TextStyle};

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    stdout: Stdout,
    text_style: TextStyle,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface, Box<dyn Error>> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, MoveTo(0, 0))?;
        enable_raw_mode()?;
        Ok(TerminalInterface {
            stdout,
            text_style: TextStyle::new(),
        })
    }

    /// Convert LF newlines to CRLF newlines, as required in Crossterm's alternate screen mode.
    fn convert_newlines(&self, input: String) -> String {
        input.replace("\n", "\n\r")
    }
}

impl Drop for TerminalInterface {
    /// Restore the terminal to its previous state when exiting.
    fn drop(&mut self) {
        execute!(self.stdout, LeaveAlternateScreen).unwrap();
        disable_raw_mode();
    }
}

impl Interface for TerminalInterface {
    fn print(&mut self, string: &str) -> Result<(), Box<dyn Error>> {
        let (width, _) = size()?;
        let wrapped = self.convert_newlines(fill(string, width as usize));
        queue!(self.stdout, Print(wrapped))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn done(&mut self) -> Result<(), Box<dyn Error>> {
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
