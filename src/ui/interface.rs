use std::error::Error;

pub trait Interface {
    /// Print text to the UI
    fn print(&mut self, text: &str) -> Result<(), Box<dyn Error>>;
    
    /// The game exited successfully, show a message then quit
    fn done(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Close the UI immediately.
    fn quit(&mut self);
}
