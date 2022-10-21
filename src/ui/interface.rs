//mod echo;
mod terminal;
//pub use echo::EchoInterface;
pub use terminal::TerminalInterface;

use crate::game::Result;

use crate::game::InputCode;

#[derive(Debug, Clone, Copy)]
pub enum ClearMode {
    Full,
    FullUnsplit,
    Single(u16),
}

/// The user interface. Responsible for both rendering the game and recieving input.
pub trait Interface {
    /// Print text to the UI
    fn print(&mut self, text: &str) -> Result<()>;

    /// Print a single character to the UI
    fn print_char(&mut self, text: char) -> Result<()>;

    /// Clear the entire window
    fn clear(&mut self, mode: ClearMode) -> Result<()>;

    /// The game exited successfully, show a message then quit
    fn done(&mut self) -> Result<()>;

    /// Set the text style to bold
    fn text_style_bold(&mut self) -> Result<()>;

    /// Set the text style to emphais (italics)
    fn text_style_emphasis(&mut self) -> Result<()>;

    /// Set the text style to reverse video.
    fn text_style_reverse(&mut self) -> Result<()>;

    /// Set the text style to fixed-width
    fn text_style_fixed(&mut self) -> Result<()>;

    /// Remove all text styles
    fn text_style_clear(&mut self) -> Result<()>;

    fn set_z_machine_version(&mut self, version: u8);

    fn read_line(&mut self, max_chars: usize) -> Result<String>;

    fn read_char(&mut self) -> Result<InputCode>;

    fn split_screen(&mut self, split: u16) -> Result<()>;

    fn get_screen_size(&self) -> (u16, u16);

    fn set_active(&mut self, active: u16) -> Result<()>;

    fn set_cursor(&mut self, line: u16, column: u16) -> Result<()>;

    fn buffer_mode(&mut self, enable: bool) -> Result<()>;

    /// Close the UI immediately.
    fn quit(&mut self);
}
