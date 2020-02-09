use std::error::Error;
use std::io::{self, Stdout, Write};

use crossterm::{
    self, execute, queue, AlternateScreen, Goto, Output, TerminalCursor, TerminalInput,
};
use log::info;
use textwrap::fill;

use crate::ui::{Interface, TextStyle};

/// A traditional terminal-based user interface.
pub struct TerminalInterface {
    _alt_screen: AlternateScreen,
    terminal: crossterm::Terminal,
    input: TerminalInput,
    stdout: Stdout,
    cursor: TerminalCursor,
    text_style: TextStyle,
}

impl TerminalInterface {
    pub fn new() -> Result<TerminalInterface, Box<dyn Error>> {
        let mut stdout = io::stdout();
        let input = crossterm::input();
        let _alt_screen = AlternateScreen::to_alternate(true)?;
        execute!(stdout, Goto(0, 0))?;
        Ok(TerminalInterface {
            _alt_screen,
            input,
            stdout,
            text_style: TextStyle::new(),
            terminal: crossterm::terminal(),
            cursor: crossterm::cursor(),
        })
    }

    /// Convert LF newlines to CRLF newlines, as required in Crossterm's alternate screen mode.
    fn convert_newlines(&self, input: String) -> String {
        input.replace("\n", "\n\r")
    }
}

impl Interface for TerminalInterface {
    fn print(&mut self, string: &str) -> Result<(), Box<dyn Error>> {
        let (width, _) = self.terminal.size()?;
        let wrapped = self.convert_newlines(fill(string, width as usize));
        queue!(self.stdout, Output(wrapped))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn done(&mut self) -> Result<(), Box<dyn Error>> {
        queue!(self.stdout, Output("\n\r[Hit any key to exit...]".into()))?;
        self.stdout.flush()?;
        self.input.read_char()?;
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
