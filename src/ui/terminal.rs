use std::error::Error;
use std::thread::sleep_ms;
use std::io::{self, Stdout, Write};

use crossterm::{self, execute, Goto, Output, AlternateScreen, TerminalInput, TerminalCursor};

use crate::ui::Interface;

pub struct Terminal {
    _alt_screen: AlternateScreen,
    input: TerminalInput,
    stdout: Stdout,
    cursor: TerminalCursor,
}

impl Terminal {
    pub fn new() -> Result<Terminal, Box<dyn Error>> {
        let mut stdout = io::stdout();
        let input = crossterm::input();
        let _alt_screen = AlternateScreen::to_alternate(true)?;
        execute!(stdout, Goto(0,0))?;
        Ok(Terminal {
            _alt_screen,
            input,
            stdout,
            cursor: crossterm::cursor(),
        })
    }
    
    fn carriage_return(&mut self) -> Result<(), Box<dyn Error>>{
        let (_, y) = self.cursor.pos()?;
        execute!(self.stdout, Goto(0, y))?;
        Ok(())
    }
}

impl Interface for Terminal {
    fn print(&mut self, string: &str) -> Result<(), Box<dyn Error>> {
        execute!(self.stdout, Output(string.into()))?;
        self.carriage_return()?;
        Ok(())
    }
    
    fn done(&mut self) -> Result<(), Box<dyn Error>> {
        execute!(self.stdout, Output("[Hit any key to exit...]".into()))?;
        self.input.read_char();
        Ok(())
    }
    
    fn quit(&mut self) {}
}
