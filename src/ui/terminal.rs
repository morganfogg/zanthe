use crate::ui::Interface;

pub struct Terminal {

}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {}
    }
}

impl Interface for Terminal {
    fn print(&self, string: &str) {
        println!("{}", string);
    }
}
