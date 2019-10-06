use std::error::Error;
use std::vec::Vec;

use log::info;

use crate::game::error::GameError;
use crate::game::instruction::{
    Context, Form, Instruction, InstructionSet, OpCode, Operand, Result as InstructionResult,
};
use crate::game::memory::Memory;
use crate::game::stack::{CallStack, StackFrame};
use crate::ui::Interface;

pub struct GameState<'a> {
    memory: Memory,
    checksum_valid: bool,
    version: u8,
    instruction_set: InstructionSet,
    call_stack: CallStack,
    interface: &'a mut dyn Interface,
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
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.call_stack.push(StackFrame::new(
            self.memory.program_counter_starts().into(),
            Vec::new(),
            None,
        ));
        loop {
            let frame = self.call_stack.frame();
            let mut code_byte = self.memory.read_byte(&mut frame.pc);
            let mut operands: Vec<Operand> = vec![];
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

            match form {
                Form::Short => {
                    if ((code_byte >> 4) & 0b11) != 3 {
                        operands.push(
                            self.memory
                                .read_operand_other(&mut frame.pc, (code_byte >> 4) & 0b11),
                        );
                    }
                }
                Form::Variable if self.version >= 5 && (code_byte == 236 || code_byte == 250) => {
                    let op_types = self.memory.read_word(&mut frame.pc);
                    operands = (0..=12)
                        .rev()
                        .step_by(2)
                        .map(|x| {
                            self.memory.read_operand_other(
                                &mut self.call_stack.frame().pc,
                                ((op_types >> x) & 0b11) as u8,
                            )
                        })
                        .collect()
                }
                Form::Variable | Form::Extended => {
                    let op_types = self.memory.read_byte(&mut frame.pc);
                    operands = (0..=6)
                        .rev()
                        .step_by(2)
                        .map(|x| {
                            self.memory.read_operand_other(
                                &mut self.call_stack.frame().pc,
                                (op_types >> x) & 0b11,
                            )
                        })
                        .collect();
                }
                Form::Long => {
                    for x in (5..=6).rev() {
                        operands.push(
                            self.memory
                                .read_operand_long(&mut frame.pc, (code_byte >> x) & 1),
                        );
                    }
                }
            }

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
            info!("{}", op_code);
            let instruction = self.instruction_set.get(&op_code).ok_or_else(|| {
                GameError::InvalidOperation(format!("Illegal opcode \"{}\"", &op_code))
            })?;

            let frame = self.call_stack.frame();

            let result = match instruction {
                Instruction::Normal(f) => {
                    let context = Context::new(frame, &mut self.memory, self.interface);
                    f(context, operands)
                }
                Instruction::Branch(f) => {
                    let condition = self.memory.get_byte(frame.pc) >> 7 == 1;
                    let label = match self.memory.get_byte(frame.pc) >> 6 & 1 {
                        0 => (self.memory.read_byte(&mut frame.pc) & 0x3f) as u16,
                        1 => self.memory.read_word(&mut frame.pc) & 0x3fff,
                        _ => unreachable!(),
                    };
                    let context = Context::new(frame, &mut self.memory, self.interface);
                    f(context, operands, condition, label)
                }
                Instruction::Store(f) => {
                    let store_to = self.memory.read_byte(&mut frame.pc);
                    let context = Context::new(frame, &mut self.memory, self.interface);
                    f(context, operands, store_to)
                }
                Instruction::StringLiteral(f) => {
                    let string = self.memory.read_string(&mut frame.pc).map_err(|e| {
                        GameError::InvalidOperation(format!("Error reading string literal: {}", e))
                    })?;
                    let context = Context::new(frame, &mut self.memory, self.interface);
                    f(context, string)
                }
            }?;
            match result {
                InstructionResult::Continue => {}
                InstructionResult::Quit => return Ok(()),
                InstructionResult::Return(result) => {
                    let old_frame = self.call_stack.pop()?;
                    if let Some(store_to) = old_frame.store_to {
                        let mut context =
                            Context::new(self.call_stack.frame(), &mut self.memory, self.interface);
                        context.set_variable(store_to, result);
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
                    if self.version < 5 {
                        for i in 0..local_count {
                            locals[i] = self.memory.read_word(&mut address);
                        }
                    }
                    if let Some(arguments) = arguments {
                        locals.splice(..arguments.len(), arguments.iter().cloned());
                    }
                    self.call_stack
                        .push(StackFrame::new(address, locals, store_to));
                }
            }
        }
    }
}
