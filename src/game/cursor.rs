use crate::game::memory::Memory;
use std::borrow::Borrow;

pub struct Cursor<T>
where
    T: Borrow<Memory>,
{
    memory: T,
    cursor: usize,
}

impl<T> Cursor<T>
where
    T: Borrow<Memory>,
{
    pub fn new(memory: T, start: usize) -> Cursor<T> {
        Cursor {
            memory: memory,
            cursor: start,
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        let result = self.memory.borrow().get_byte(self.cursor);
        self.cursor += 1;
        result
    }

    pub fn read_word(&mut self) -> u16 {
        let result = self.memory.borrow().get_word(self.cursor);
        self.cursor += 2;
        result
    }
}

impl<'a, T> Cursor<T>
where
    T: Borrow<Memory> + AsMut<Memory>,
{
    pub fn write_byte(&mut self, content: u8) {
        self.memory.as_mut().write_byte(self.cursor, content);
        self.cursor += 1;
    }
    pub fn write_word(&mut self, content: u16) {
        self.memory.as_mut().write_word(self.cursor, content);
        self.cursor += 2;
    }
}
