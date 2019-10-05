use std::error::Error;
use std::io::{self, Stdout, Write};

use crossterm::{
    self, execute, queue, AlternateScreen, Goto, Output, TerminalCursor, TerminalInput,
};
use log::info;
use textwrap::fill;

use crate::ui::Interface;

pub struct Terminal {
    _alt_screen: AlternateScreen,
    terminal: crossterm::Terminal,
    input: TerminalInput,
    stdout: Stdout,
    cursor: TerminalCursor,
}

impl Terminal {
    pub fn new() -> Result<Terminal, Box<dyn Error>> {
        let mut stdout = io::stdout();
        let input = crossterm::input();
        let _alt_screen = AlternateScreen::to_alternate(true)?;
        execute!(stdout, Goto(0, 0))?;
        Ok(Terminal {
            _alt_screen,
            input,
            stdout,
            terminal: crossterm::terminal(),
            cursor: crossterm::cursor(),
        })
    }

    fn convert_newlines(&self, input: String) -> String {
        input.replace("\n", "\n\r")
    }
}

impl Interface for Terminal {
    fn print(&mut self, string: &str) -> Result<(), Box<dyn Error>> {
        let (width, _) = self.terminal.size()?;
        info!("{}", width);
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

    fn quit(&mut self) {}
}
