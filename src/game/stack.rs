use std::error::Error;

use crate::game::error::GameError;

pub struct CallStack {
    frames: Vec<StackFrame>,
}

pub struct StackFrame {
    pub pc: usize,
    pub stack: Vec<u16>,
    pub locals: Vec<u16>,
    pub store_to: Option<u8>,
}

impl StackFrame {
    pub fn new(pc: usize, locals: Vec<u16>, store_to: Option<u8>) -> StackFrame {
        StackFrame {
            stack: Vec::new(),
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

    pub fn pop_stack(&mut self) -> Result<u16, Box<dyn Error>> {
        self.stack.pop().ok_or_else(|| {
            GameError::InvalidOperation("Attempted to read from empty stack".into()).into()
        })
    }

    pub fn push_stack(&mut self, value: u16) {
        self.stack.push(value)
    }
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack { frames: Vec::new() }
    }

    pub fn frame(&mut self) -> &mut StackFrame {
        let top = self.frames.len() - 1;
        &mut self.frames[top]
    }

    pub fn push(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Result<StackFrame, Box<dyn Error>> {
        if self.frames.len() <= 1 {
            Err(GameError::InvalidOperation("Tried to return from main routine".into()).into())
        } else {
            Ok(self.frames.pop().unwrap())
        }
    }
}