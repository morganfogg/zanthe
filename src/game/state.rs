use std::error::Error;
use std::vec::Vec;

use log::debug;
use rand::{rngs::StdRng, SeedableRng};

use crate::game::error::GameError;
use crate::game::instruction::{
    Form, Instruction, InstructionSet, OpCode, Operand, OperandSet, Result as InstructionResult,
};
use crate::game::memory::Memory;
use crate::game::stack::{CallStack, StackFrame};
use crate::ui::Interface;

/// Represents the current state of play.
pub struct GameState<'a> {
    pub memory: Memory,
    pub checksum_valid: bool,
    pub version: u8,
    pub instruction_set: InstructionSet,
    call_stack: CallStack,
    pub interface: &'a mut dyn Interface,
    pub rng: StdRng,
}

impl<'a> GameState<'a> {
    pub fn new(data: Vec<u8>, interface: &'a mut dyn Interface) -> Result<GameState, GameError> {
        let memory = Memory::new(data);
        memory.validate_header()?;
        Ok(GameState {
            checksum_valid: memory.verify(),
            version: memory.version(),
            instruction_set: InstructionSet::new(memory.version()),
            call_stack: CallStack::new(),
            memory,
            interface,
            rng: StdRng::from_entropy(),
        })
    }

    pub fn frame(&mut self) -> &mut StackFrame {
        self.call_stack.frame()
    }

    /// Start the game
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.call_stack.push(StackFrame::new(
            self.memory.program_counter_starts().into(),
            Vec::new(),
            0,
            None,
        ));
        loop {
            let frame = self.call_stack.frame();
            debug!("--------------------------------------");
            debug!("PC AT {:x}", frame.pc);
            let mut code_byte = self.memory.read_byte(&mut frame.pc);
            let mut operands: Vec<Operand> = Vec::new();
            let form;
            if code_byte == 190 {
                form = Form::Extended;
                code_byte = self.memory.read_byte(&mut frame.pc);
            } else {
                form = match code_byte >> 6 {
                    0b11 => Form::Variable,
                    0b10 => Form::Short,
                    _ => Form::Long,
                };
            }
            let mut pc = frame.pc;
            match form {
                Form::Short => {
                    if ((code_byte >> 4) & 0b11) != 3 {
                        operands.push(
                            self.memory
                                .read_operand_other(&mut pc, (code_byte >> 4) & 0b11),
                        );
                    }
                }
                Form::Variable if self.version >= 5 && (code_byte == 236 || code_byte == 250) => {
                    let op_types = self.memory.read_word(&mut pc);
                    operands = (0..=14)
                        .rev()
                        .step_by(2)
                        .map(|x| {
                            self.memory
                                .read_operand_other(&mut pc, ((op_types >> x) & 0b11) as u8)
                        })
                        .collect()
                }
                Form::Variable | Form::Extended => {
                    let op_types = self.memory.read_byte(&mut pc);
                    operands = (0..=6)
                        .rev()
                        .step_by(2)
                        .map(|x| {
                            self.memory
                                .read_operand_other(&mut pc, (op_types >> x) & 0b11)
                        })
                        .collect();
                }
                Form::Long => {
                    for x in (5..=6).rev() {
                        operands.push(self.memory.read_operand_long(&mut pc, (code_byte >> x) & 1));
                    }
                }
            }

            self.call_stack.frame().pc = pc;

            let operands = OperandSet::new(operands);

            let op_code = match form {
                Form::Long => OpCode::TwoOp(code_byte & 31),
                Form::Extended => OpCode::Extended(code_byte),
                Form::Short => {
                    if ((code_byte >> 4) & 3) == 3 {
                        OpCode::ZeroOp(code_byte & 15)
                    } else {
                        OpCode::OneOp(code_byte & 15)
                    }
                }
                Form::Variable => {
                    if ((code_byte >> 5) & 1) == 0 {
                        OpCode::TwoOp(code_byte & 31)
                    } else {
                        OpCode::VarOp(code_byte & 31)
                    }
                }
            };
            let instruction = self.instruction_set.get(&op_code).ok_or_else(|| {
                GameError::InvalidOperation(format!("Illegal opcode \"{}\"", &op_code))
            })?;

            let frame = self.frame();
            let mut pc = frame.pc;

            let result = match instruction {
                Instruction::Normal(f, name) => {
                    debug!("{} {} {:?}", op_code, name, operands);
                    f(self, operands)
                }
                Instruction::Branch(f, name) => {
                    let condition = self.memory.get_byte(pc) >> 7 == 1;
                    let offset = if self.memory.get_byte(pc) >> 6 & 1 == 1 {
                        // The offset is an unsigned 6-bit number.
                        (self.memory.read_byte(&mut pc) & 0x3f) as i16
                    } else {
                        // The offset is a signed 14-bit number.
                        let base = self.memory.read_word(&mut pc);
                        if base >> 13 == 1 {
                            -((base & 0x1fff) as i16)
                        } else {
                            (base & 0x1fff) as i16
                        }
                    };
                    debug!(
                        "{} {} {:?} IF {} OFFSET {}",
                        op_code, name, operands, condition, offset
                    );
                    self.frame().pc = pc;
                    f(self, operands, condition, offset)
                }
                Instruction::Store(f, name) => {
                    let store_to = self.memory.read_byte(&mut pc);
                    debug!("{} {} {:?} STORE {:x}", op_code, name, operands, store_to);
                    self.frame().pc = pc;
                    f(self, operands, store_to)
                }
                Instruction::BranchStore(f, name) => {
                    let store_to = self.memory.read_byte(&mut pc);
                    let condition = self.memory.get_byte(pc) >> 7 == 1;

                    let offset = if self.memory.get_byte(pc) >> 6 & 1 == 1 {
                        // The offset is an unsigned 6-bit number.
                        (self.memory.read_byte(&mut pc) & 0x3f) as i16
                    } else {
                        // The offset is a signed 14-bit number.
                        let base = self.memory.read_word(&mut pc);
                        if base >> 13 == 1 {
                            -((base & 0x1fff) as i16)
                        } else {
                            (base & 0x1fff) as i16
                        }
                    };
                    debug!(
                        "{} {} {:?} STORE {} IF {} OFFSET {}",
                        op_code, name, operands, store_to, condition, offset
                    );
                    self.frame().pc = pc;
                    f(self, operands, condition, offset, store_to)
                }
                Instruction::StringLiteral(f, name) => {
                    let string = self.memory.read_string(&mut pc).map_err(|e| {
                        GameError::InvalidOperation(format!("Error reading string literal: {}", e))
                    })?;
                    debug!("{} {} {:?}", op_code, name, string);
                    self.frame().pc = pc;
                    f(self, string)
                }
            }?;

            match result {
                InstructionResult::Continue => {}
                InstructionResult::Quit => return Ok(()),
                InstructionResult::Return(result) => {
                    let old_frame = self.call_stack.pop()?;
                    if let Some(store_to) = old_frame.store_to {
                        self.set_variable(store_to, result);
                    }
                }
                InstructionResult::Invoke {
                    mut address,
                    store_to,
                    arguments,
                } => {
                    let local_count = self.memory.read_byte(&mut address) as usize;
                    if local_count > 15 {
                        return Err(GameError::InvalidOperation(
                            "Routine tried to create more than 15 locals".into(),
                        )
                        .into());
                    }
                    let mut locals = vec![0; local_count];

                    // In z4 and earlier, locals can have default values. In z5 and later,
                    // locals always default to zero.
                    if self.version < 5 {
                        for i in 0..local_count {
                            locals[i] = self.memory.read_word(&mut address);
                        }
                    }

                    let mut arg_count = 0;

                    if let Some(arguments) = arguments {
                        locals.splice(..arguments.len(), arguments.iter().cloned());
                        arg_count = arguments.len();
                    }
                    self.call_stack
                        .push(StackFrame::new(address, locals, arg_count, store_to));
                }
            }
        }
    }
    pub fn set_variable(&mut self, variable: u8, value: u16) {
        match variable {
            0x0 => {
                debug!("SET SP = {0} [{0:x}]", value);
                self.frame().push_stack(value)
            }
            0x1..=0xf => {
                debug!("SET L{:x} = {1} [{1:x}]", variable - 0x1, value);
                self.frame().set_local(variable as usize - 1, value);
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
                .frame()
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
                .frame()
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
                result = self.frame().pop_stack();
                debug!(
                    "GET SP = {}",
                    match result {
                        Ok(v) => format!("{0}, [{0:x}]", v),
                        Err(_) => "ERROR".to_string(),
                    }
                );
            }
            0x1..=0xf => {
                let local = self.frame().get_local(variable as usize - 0x1);
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
