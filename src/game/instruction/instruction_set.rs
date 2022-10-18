mod common;
mod version_gte3;
mod version_gte4;
mod version_gte5;

use crate::game::instruction::{Instruction, OpCode};

/// Represents all the instructions available to the Z-Machine version specified in the game file.
pub struct InstructionSet {
    instructions: Vec<Option<Instruction>>,
}

impl InstructionSet {
    pub fn new(version: u8) -> InstructionSet {
        let mut instructions: Vec<Option<Instruction>> = vec![None; 512];
        for (code, instruction) in common::instructions() {
            instructions[code.lookup_value()] = Some(instruction);
        }

        if version >= 3 {
            for (code, instruction) in version_gte3::instructions() {
                instructions[code.lookup_value()] = Some(instruction);
            }
        }

        if version >= 4 {
            for (code, instruction) in version_gte4::instructions() {
                instructions[code.lookup_value()] = Some(instruction);
            }
        }

        if version >= 5 {
            for (code, instruction) in version_gte5::instructions() {
                instructions[code.lookup_value()] = Some(instruction);
            }
        }

        InstructionSet { instructions }
    }

    pub fn get(&self, opcode: &OpCode) -> Option<Instruction> {
        self.instructions[opcode.lookup_value()].clone()
    }
}
