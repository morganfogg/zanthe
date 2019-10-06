use std::error::Error;

use crate::ui::interface::Interface;

/// A UI module that does nothing. For testing purposes.
pub struct NullInterface {}

impl NullInterface {
    pub fn new() -> NullInterface {
        NullInterface {}
    }
}

impl Interface for NullInterface {
    fn print(&mut self, text: &str) -> Result<(), Box<dyn Error>> {
        println!("Print called with '{}'", text);
        Ok(())
    }

    /// The game exited successfully, show a message then quit
    fn done(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Done");
        Ok(())
    }

    /// Close the UI immediately.
    fn quit(&mut self) {
        println!("Quit");
    }
}
