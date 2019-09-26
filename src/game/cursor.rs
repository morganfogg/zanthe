use std::borrow::Borrow;
use std::convert::TryFrom;
use std::io::{Error as IOError, ErrorKind, Seek, SeekFrom};

use crate::game::memory::Memory;

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

    pub fn tell(&self) -> usize {
        self.cursor
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

    pub fn inner(&self) -> &Memory {
        return self.memory.borrow();
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

impl<T> Seek for Cursor<T>
where
    T: Borrow<Memory>,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, IOError> {
        let mut new_cursor = self.cursor;
        match pos {
            SeekFrom::Start(x) => match usize::try_from(x) {
                Ok(x) => {
                    new_cursor = x;
                }
                Err(_) => {
                    return Err(IOError::new(ErrorKind::Other, "Integer overflow"));
                }
            },
            SeekFrom::Current(x) => match usize::try_from(x) {
                Ok(x) => match new_cursor.checked_add(x) {
                    Some(x) => {
                        new_cursor = x;
                    }
                    None => {
                        return Err(IOError::new(ErrorKind::UnexpectedEof, "Seek out of bounds"));
                    }
                },
                Err(_) => {
                    return Err(IOError::new(ErrorKind::Other, "Integer overflow"));
                }
            },
            SeekFrom::End(x) => {
                if x > 0 {
                    return Err(IOError::new(ErrorKind::UnexpectedEof, "Seek out of bounds"));
                } else {
                    match usize::try_from(-x) {
                        Ok(x) => match self.memory.borrow().data_length().checked_sub(x) {
                            Some(x) => {
                                new_cursor = x;
                            }
                            None => {
                                return Err(IOError::new(
                                    ErrorKind::UnexpectedEof,
                                    "Seek out of bounds",
                                ));
                            }
                        },
                        Err(_) => {
                            return Err(IOError::new(ErrorKind::Other, "Integer overflow"));
                        }
                    }
                }
            }
        }
        if new_cursor >= self.memory.borrow().data_length() {
            return Err(IOError::new(ErrorKind::UnexpectedEof, "Seek out of bounds"));
        }
        self.cursor = new_cursor;
        Ok(self.cursor as u64)
    }
}