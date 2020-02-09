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

    fn text_style_bold(&mut self) {
        println!("Text style set to BOLD");
    }

    fn text_style_emphasis(&mut self) {
        println!("Text style set to EMPHASIS");
    }

    fn text_style_reverse(&mut self) {
        println!("Text style set to REVERSE VIDEO");
    }

    fn text_style_fixed(&mut self) {
        println!("Text style set to FIXED WIDTH");
    }

    fn text_style_clear(&mut self) {
        println!("Text style CLEARED");
    }

    /// Close the UI immediately.
    fn quit(&mut self) {
        println!("Quit");
    }
}
