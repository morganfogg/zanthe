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

struct UndoBufferEntry {
    pub memory: Memory,
    pub call_stack: CallStack,
    pub rng: StdRng,
}

/// Represents the current state of play.
pub struct GameState<'a> {
    pub memory: Memory,
    pub checksum_valid: bool,
    pub version: u8,
    pub instruction_set: InstructionSet,
    pub interface: &'a mut dyn Interface,
    pub rng: StdRng,
    call_stack: CallStack,
    undo_buffer: Vec<UndoBufferEntry>,
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
            undo_buffer: Vec::new(),
            rng: StdRng::from_entropy(),
            memory,
            interface,
        })
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
            match self.next_op()? {
                InstructionResult::Continue => {}
                InstructionResult::Quit => return Ok(()),
                InstructionResult::Return(result) => self.return_with(result)?,
                InstructionResult::Invoke {
                    address,
                    store_to,
                    arguments,
                } => self.invoke(address, store_to, arguments)?,
            }
        }
    }

    pub fn frame(&mut self) -> &mut StackFrame {
        self.call_stack.frame()
    }

    pub fn save_undo(&mut self, restore_flag: u8) {
        self.set_variable(restore_flag, 2);
        self.undo_buffer.push(UndoBufferEntry {
            memory: self.memory.clone(),
            call_stack: self.call_stack.clone(),
            rng: self.rng.clone(),
        });

        self.poke_variable(restore_flag, 1).unwrap();
    }

    pub fn restore_undo(&mut self) -> bool {
        if let Some(buffer) = self.undo_buffer.pop() {
            self.memory = buffer.memory;
            self.call_stack = buffer.call_stack;
            self.rng = buffer.rng;
            true
        } else {
            false
        }
    }

    fn next_op(&mut self) -> Result<InstructionResult, Box<dyn Error>> {
        let frame = self.call_stack.frame();
        debug!("--------------------------------------");
        debug!("PC AT {:x}", frame.pc);
        let mut code_byte = self.memory.read_byte(&mut frame.pc);
        let mut operands: Vec<Operand> = Vec::new();

        // Determine the form of the instruction.
        let form = if code_byte == 190 {
            code_byte = self.memory.read_byte(&mut frame.pc);
            Form::Extended
        } else {
            match code_byte >> 6 {
                0b11 => Form::Variable,
                0b10 => Form::Short,
                _ => Form::Long,
            }
        };

        let mut pc = frame.pc;

        // Read the op code
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

        // Read in the instruction's operands.
        match form {
            Form::Short => {
                if let OpCode::OneOp(_) = op_code {
                    let operand = self
                        .memory
                        .read_operand_other(&mut pc, (code_byte >> 4) & 3);
                    operands.push(operand);
                }
            }
            Form::Variable if self.version >= 5 && (code_byte == 236 || code_byte == 250) => {
                let op_types = self.memory.read_word(&mut pc);
                operands = (0..=14)
                    .rev()
                    .step_by(2)
                    .map(|x| {
                        self.memory
                            .read_operand_other(&mut pc, ((op_types >> x) & 3) as u8)
                    })
                    .collect()
            }
            Form::Variable | Form::Extended => {
                let op_types = self.memory.read_byte(&mut pc);
                operands = (0..=6)
                    .rev()
                    .step_by(2)
                    .map(|x| self.memory.read_operand_other(&mut pc, (op_types >> x) & 3))
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

        let instruction = self.instruction_set.get(&op_code).ok_or_else(|| {
            GameError::InvalidOperation(format!("Illegal opcode \"{}\"", &op_code))
        })?;

        let frame = self.frame();
        let mut pc = frame.pc;

        match instruction {
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
                        -1i16 & (base & 0x1fff) as i16
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
                        -1i16 & (base & 0x1fff) as i16
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
        }
    }

    /// Move game control into a subroutine.
    fn invoke(
        &mut self,
        mut address: usize,
        store_to: Option<u8>,
        arguments: Option<Vec<u16>>,
    ) -> Result<(), Box<dyn Error>> {
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
            for local in locals.iter_mut() {
                *local = self.memory.read_word(&mut address);
            }
        }

        let mut arg_count = 0;

        if let Some(arguments) = arguments {
            locals.splice(..arguments.len(), arguments.iter().cloned());
            arg_count = arguments.len();
        }
        self.call_stack
            .push(StackFrame::new(address, locals, arg_count, store_to));
        Ok(())
    }

    // Return control from a subroutine to its calling routine.
    pub fn return_with(&mut self, result: u16) -> Result<(), Box<dyn Error>> {
        let old_frame = self.call_stack.pop()?;
        if let Some(store_to) = old_frame.store_to {
            self.set_variable(store_to, result);
        }
        Ok(())
    }

    /// Invoke an interupt routine and return the result of that routine.
    pub fn run_routine(&mut self, address: u16) -> Result<Option<u16>, Box<dyn Error>> {
        self.call_stack
            .push(StackFrame::new(address as usize, Vec::new(), 0, None));

        let starting_depth = self.call_stack.depth();

        loop {
            match self.next_op()? {
                InstructionResult::Continue => {}
                InstructionResult::Quit => return Ok(None),
                InstructionResult::Return(result) => {
                    if self.call_stack.depth() == starting_depth {
                        return Ok(Some(result));
                    } else {
                        self.return_with(result)?;
                    }
                }
                InstructionResult::Invoke {
                    address,
                    store_to,
                    arguments,
                } => self.invoke(address, store_to, arguments)?,
            }
        }
    }

    /// Set a game variable
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

    /// Retrieve a game varaible.
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
