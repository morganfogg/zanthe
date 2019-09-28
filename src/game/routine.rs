use crate::game::cursor::Cursor;
use crate::game::error::GameError;
use crate::game::instruction::{Instruction, InstructionResult, InstructionSet};
use crate::game::memory::Memory;
use crate::game::operand::Operand;

enum InstructionForm {
    Long,
    Short,
    Extended,
    Variable,
}

pub struct Routine<'a> {
    stack: Vec<u8>,
    variables: Vec<u8>,
    version: u8,
    cursor: &'a mut Cursor<&'a Memory>,
    pub instruction_set: &'a InstructionSet,
}

impl<'a> Routine<'a> {
    pub fn new(
        cursor: &'a mut Cursor<&'a Memory>,
        instruction_set: &'a InstructionSet,
    ) -> Routine<'a> {
        Routine {
            stack: Vec::new(),
            variables: Vec::new(),
            version: cursor.inner().version(),
            instruction_set,
            cursor,
        }
    }

    fn read_operand_long(&mut self, op_type: u8) -> Operand {
        match op_type {
            0 => Operand::SmallConstant(self.cursor.read_byte()),
            1 => Operand::Variable(self.cursor.read_byte()),
            _ => unreachable!(),
        }
    }

    fn read_operand_other(&mut self, op_type: u8) -> Operand {
        match op_type {
            0 => Operand::LargeConstant(self.cursor.read_word()),
            1 => Operand::SmallConstant(self.cursor.read_byte()),
            2 => Operand::Variable(self.cursor.read_byte()),
            3 => Operand::Omitted,
            _ => unreachable!(),
        }
    }

    pub fn memory(&self) -> &Memory {
        self.cursor.inner()
    }

    pub fn invoke(&mut self) -> Result<(), GameError> {
        for _ in 1..=2 {
            let next = self.next();
            println!("{:?}", next);
        }
        Ok(())
    }

    fn next(&mut self) -> InstructionResult {
        println!("{:x}", self.cursor.tell());
        let mut code = self.cursor.read_byte();
        let form;
        let mut operands: Vec<Operand> = vec![];
        if code == 190 {
            form = InstructionForm::Extended;
            code = self.cursor.read_byte();
        } else {
            form = match code >> 6 {
                0b11 => InstructionForm::Variable,
                0b10 => InstructionForm::Short,
                _ => InstructionForm::Long,
            };
        }
        match form {
            InstructionForm::Short => {
                if code >> 4 == 3 {
                    operands.push(self.read_operand_other((code >> 4) & 0b11));
                }
            }
            InstructionForm::Variable if self.version >= 5 && (code == 236 || code == 250) => {
                let op_types = self.cursor.read_word();
                operands = (0..=12)
                    .rev()
                    .step_by(2)
                    .map(|x| self.read_operand_other(((op_types >> x) & 0b11) as u8))
                    .collect()
            }
            InstructionForm::Variable | InstructionForm::Extended => {
                let op_types = self.cursor.read_byte();
                operands = (0..=6)
                    .rev()
                    .step_by(2)
                    .map(|x| self.read_operand_other((op_types >> x) & 0b11))
                    .collect();
            }
            InstructionForm::Long => {
                for x in 6..=5 {
                    operands.push(self.read_operand_long((code >> x) & 0b1));
                }
            }
        }

        let instruction = self.instruction_set.get(code);
        let instruction = match instruction {
            Some(i) => i,
            None => return InstructionResult::Error("Illegal opcode".into()),
        };
        match instruction {
            Instruction::Normal(f) => f(self, operands),
            Instruction::Branch(f) => {
                let condition = self.cursor.peek_byte() >> 7 == 1;
                let label = match self.cursor.peek_byte() >> 6 & 1 {
                    0 => (self.cursor.read_byte() & 0x3f) as u16,
                    1 => self.cursor.read_word() & 0x3fff,
                    _ => unreachable!(),
                };
                f(self, operands, condition, label)
            }
            Instruction::Return(f) => {
                let variable = self.cursor.read_byte();
                f(self, operands, variable)
            }
            Instruction::StringLiteral(f) => {
                let string = match self.cursor.read_string() {
                    Ok(v) => v,
                    Err(e) => {
                        return InstructionResult::Error(
                            format!("Error reading string literal: {}", e).into(),
                        )
                    }
                };
                f(self, string)
            }
        }
    }
}
