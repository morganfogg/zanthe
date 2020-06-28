//mod echo;
mod terminal;
//pub use echo::EchoInterface;
pub use terminal::TerminalInterface;

use anyhow::Result;

use crate::game::InputCode;

/// The user interface. Responsible for both rendering the game and recieving input.
pub trait Interface {
    /// Print text to the UI
    fn print(&mut self, text: &str) -> Result<()>;

    /// Print a single character to the UI
    fn print_char(&mut self, text: char) -> Result<()>;

    /// Clear the entire window
    fn clear(&mut self) -> Result<()>;

    /// The game exited successfully, show a message then quit
    fn done(&mut self) -> Result<()>;

    /// Set the text style to bold
    fn text_style_bold(&mut self);

    /// Set the text style to emphais (italics)
    fn text_style_emphasis(&mut self);

    /// Set the text style to reverse video.
    fn text_style_reverse(&mut self);

    /// Set the text style to fixed-width
    fn text_style_fixed(&mut self);

    /// Remove all text styles
    fn text_style_clear(&mut self);

    fn read_line(&mut self, max_chars: usize) -> Result<String>;

    fn read_char(&mut self) -> Result<InputCode>;

    /// Close the UI immediately.
    fn quit(&mut self);
}
