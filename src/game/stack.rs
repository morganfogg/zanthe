use crate::game::Result;

use crate::game::error::GameError;
use crate::game::instruction::Result as InstructionResult;

/// The call-stack of the machine. Divided into stack frames, representing individual routines.
#[derive(Clone)]
pub struct CallStack {
    frames: Vec<StackFrame>,
}

/// The section of the call stack associated with a particular routine.
#[derive(Clone)]
pub struct StackFrame {
    pub pc: usize,
    pub stack: Vec<u16>,
    pub locals: Vec<u16>,
    pub store_to: Option<u8>,
    pub arg_count: usize,
}

impl StackFrame {
    pub fn new(pc: usize, locals: Vec<u16>, arg_count: usize, store_to: Option<u8>) -> StackFrame {
        StackFrame {
            stack: Vec::new(),
            arg_count,
            locals,
            pc,
            store_to,
        }
    }

    pub fn get_local(&self, index: usize) -> u16 {
        self.locals[index]
    }

    pub fn set_local(&mut self, index: usize, value: u16) {
        self.locals[index] = value;
    }

    pub fn pop_stack(&mut self) -> Result<u16> {
        self.stack
            .pop()
            .ok_or_else(|| GameError::invalid_operation("Attempted to read from empty stack"))
    }

    pub fn push_stack(&mut self, value: u16) {
        self.stack.push(value);
    }

    pub fn branch(&mut self, offset: i16) -> InstructionResult {
        match offset {
            0..=1 => InstructionResult::Return(offset as u16),
            _ => {
                if offset < 0 {
                    self.pc -= (-offset) as usize + 2;
                } else {
                    self.pc += offset as usize - 2;
                }
                InstructionResult::Continue
            }
        }
    }

    pub fn conditional_branch(
        &mut self,
        offset: i16,
        condition: bool,
        expected: bool,
    ) -> InstructionResult {
        if condition == expected {
            self.branch(offset)
        } else {
            InstructionResult::Continue
        }
    }
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack { frames: Vec::new() }
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn frame(&mut self) -> &mut StackFrame {
        let top = self.frames.len() - 1;
        &mut self.frames[top]
    }

    pub fn push(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    pub fn throw(&mut self, to: usize) -> Result<()> {
        if self.frames.len() <= to as usize {
            return Err(GameError::invalid_operation(
                "Tried to throw to an invalid stack frame",
            ));
        }
        self.frames.truncate(to);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<StackFrame> {
        if self.frames.len() <= 1 {
            Err(GameError::invalid_operation(
                "Tried to return from main routine",
            ))
        } else {
            Ok(self.frames.pop().unwrap())
        }
    }
}
