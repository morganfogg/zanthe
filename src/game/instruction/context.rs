use std::error::Error;

use rand::rngs::StdRng;

use crate::game::memory::Memory;
use crate::game::stack::StackFrame;
use crate::ui::Interface;

/// The context represents the parts of the game state that are passed to instruction calls.
pub struct Context<'a> {
    pub frame: &'a mut StackFrame,
    pub memory: &'a mut Memory,
    pub interface: &'a mut dyn Interface,
    pub rng: &'a mut StdRng,
}

impl<'a> Context<'a> {
    pub fn new(
        frame: &'a mut StackFrame,
        memory: &'a mut Memory,
        interface: &'a mut dyn Interface,
        rng: &'a mut StdRng,
    ) -> Context<'a> {
        Context {
            frame,
            memory,
            interface,
            rng,
        }
    }

    pub fn set_variable(&mut self, variable: u8, value: u16) {
        match variable {
            0x0 => self.frame.push_stack(value),
            0x1..=0xf => {
                self.frame.set_local(variable as usize - 1, value);
            }
            _ => {
                self.memory.set_global(variable - 16, value);
            }
        }
    }

    pub fn get_variable(&mut self, variable: u8) -> Result<u16, Box<dyn Error>> {
        match variable {
            0x0 => self.frame.pop_stack(),
            0x1..=0xf => Ok(self.frame.get_local(variable as usize - 1)),
            _ => Ok(self.memory.get_global(variable - 16)),
        }
    }
}
