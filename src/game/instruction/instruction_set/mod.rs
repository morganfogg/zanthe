mod common;
mod version_gte3;
mod version_gte4;
mod version_gte5;

use std::collections::HashMap;

use crate::game::instruction::{Instruction, OpCode};

/// Represents all the instructions available to the Z-Machine version specified in the game file.
pub struct InstructionSet {
    instructions: HashMap<OpCode, Instruction>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: HashMap<OpCode, Instruction> = common::instructions();

        if version >= 3 {
            instructions.extend(version_gte3::instructions());
        }

        if version >= 4 {
            instructions.extend(version_gte4::instructions());
        }

        if version >= 5 {
            instructions.extend(version_gte5::instructions());
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: &OpCode) -> Option<Instruction> {
        self.instructions.get(opcode).cloned()
    }
}
