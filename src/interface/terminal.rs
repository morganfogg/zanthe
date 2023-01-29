mod window;

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::game::Result;
use crate::interface::{ClearMode, InputCode, Interface};
use window::{SplitDirection, SplitSize, TextStream, WindowKind, WindowManager};

pub struct TerminalInterface {
    wm: WindowManager,
    upper_screen_id: usize,
    lower_screen_id: usize,
}

impl TerminalInterface {
    pub fn new() -> Result<Self> {
        Ok(Self {
            wm: WindowManager::new()?,
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
            SplitDirection::Above,
            SplitSize::Unlimited,
            WindowKind::TextStream(TextStream::default()),
        )?;
        self.upper_screen_id = self.wm.split(
            self.lower_screen_id,
            SplitDirection::Above,
            SplitSize::Fixed(10),
            WindowKind::TextStream(TextStream::default()),
        )?;
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<()> {
        self.wm.print(text);
        Ok(())
    }

    /// Print a single character to the UI
    fn print_char(&mut self, text: char) -> Result<()> {
        self.wm.print_char(text);
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

    fn read_line(&mut self, max_chars: usize) -> Result<String> {
        use std::thread;
        use std::time::Duration;

        self.wm.render();
        thread::sleep(Duration::from_millis(4000));
        todo!();
    }

    fn read_char(&mut self) -> Result<InputCode> {
        use std::thread;
        use std::time::Duration;

        self.wm.render();
        thread::sleep(Duration::from_millis(4000));
        todo!();
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
