use std::error::Error;

use log::debug;
use rand::rngs::StdRng;

use crate::game::error::GameError;
use crate::game::memory::Memory;
use crate::game::stack::StackFrame;
use crate::ui::Interface;

/// The context represents the parts of the game state that are passed to instruction calls.
pub struct Context<'a> {
    pub frame: &'a mut StackFrame,
    pub memory: &'a mut Memory,
    pub interface: &'a mut dyn Interface,
    pub rng: &'a mut StdRng,
    pub checksum_valid: bool,
}

impl<'a> Context<'a> {
    pub fn new(
        frame: &'a mut StackFrame,
        memory: &'a mut Memory,
        interface: &'a mut dyn Interface,
        rng: &'a mut StdRng,
        checksum_valid: bool,
    ) -> Context<'a> {
        Context {
            frame,
            memory,
            interface,
            rng,
            checksum_valid,
        }
    }

    pub fn set_variable(&mut self, variable: u8, value: u16) {
        match variable {
            0x0 => {
                debug!("SET SP = {0} [{0:x}]", value);
                self.frame.push_stack(value)
            }
            0x1..=0xf => {
                debug!("SET L{:x} = {1} [{1:x}]", variable - 0x1, value);
                self.frame.set_local(variable as usize - 1, value);
            }
            _ => {
                debug!("SET G{:x} = {1} [{1:x}]", variable - 0x10, value);
                self.memory.set_global(variable - 16, value);
            }
        }
    }

    /// Used by the "indirect variable reference" opcodes. Reads a variable without potentially
    /// modifying the stack.
    pub fn peek_variable(&mut self, variable: u8) -> Result<u16, Box<dyn Error>> {
        if variable == 0 {
            Ok(*self
                .frame
                .stack
                .last()
                .ok_or_else(|| GameError::InvalidOperation("Can't edit empty stack".into()))?)
        } else {
            self.get_variable(variable)
        }
    }

    /// Used by the "indirect variable reference" opcodes. When writing the stack, replace the the
    /// topmost item in the stack instead of pushing on top of it.
    pub fn poke_variable(&mut self, variable: u8, value: u16) -> Result<(), Box<dyn Error>> {
        if variable == 0 {
            *self
                .frame
                .stack
                .last_mut()
                .ok_or_else(|| GameError::InvalidOperation("Can't edit empty stack".into()))? =
                value;
        } else {
            self.set_variable(variable, value);
        }
        Ok(())
    }

    pub fn get_variable(&mut self, variable: u8) -> Result<u16, Box<dyn Error>> {
        let result;
        match variable {
            0x0 => {
                result = self.frame.pop_stack();
                debug!(
                    "GET SP = {}",
                    match result {
                        Ok(v) => format!("{0}, [{0:x}]", v),
                        Err(_) => "ERROR".to_string(),
                    }
                );
            }
            0x1..=0xf => {
                let local = self.frame.get_local(variable as usize - 0x1);
                debug!("GET L{:x} = {1} [{1:x}]", variable - 0x1, local);
                result = Ok(local);
            }
            _ => {
                let global = self.memory.get_global(variable - 0x10);
                debug!("GET G{:x} = {1} [{1:x}]", variable - 0x10, global);
                result = Ok(global);
            }
        };
        result
    }
}
