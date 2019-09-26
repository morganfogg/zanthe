use std::fmt::{self, Debug, Display, Formatter};

use crate::game::cursor::Cursor;
use crate::game::memory::Memory;

#[derive(Debug)]
enum Instruction {
    Long(u8),
    Short(u8),
    Extended(u8),
    Variable(u8),
}

enum Operand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Operand::LargeConstant(v) => format!("LargeConstant({:x})", v),
                Operand::SmallConstant(v) => format!("SmallConstant({:x})", v),
                Operand::Variable(v) => format!("Variable({:x})", v),
                Operand::Omitted => "Omitted".to_string(),
            }
        )
    }
}

impl Debug for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

pub struct Routine<'a> {
    stack: Vec<u8>,
    variables: Vec<u8>,
    version: u8,
    cursor: &'a mut Cursor<&'a Memory>,
}

impl<'a> Routine<'a> {
    pub fn new(cursor: &'a mut Cursor<&'a Memory>) -> Routine<'a> {
        Routine {
            stack: Vec::new(),
            variables: Vec::new(),
            version: cursor.borrow().version(),
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

    pub fn invoke(&mut self) {
        println!("Okay!");
        let op = self.cursor.read_byte();

        let instruction;
        let mut operands: Vec<Operand> = vec![];
        if op == 190 {
            instruction = Instruction::Extended(self.cursor.read_byte());
        } else {
            instruction = match op >> 6 {
                0b11 => Instruction::Variable(op & 0b11111),
                0b10 => Instruction::Short(op & 0b1111),
                _ => Instruction::Long(op & 0b11111),
            };
        }
        match instruction {
            Instruction::Short(_) => {
                operands.push(self.read_operand_other((op >> 4) & 0b11));
            }
            Instruction::Variable(code) if self.version >= 5 && (code == 0xC || code == 0x1A) => {
                let op_types = self.cursor.read_word();
                operands = (0..=12)
                    .rev()
                    .step_by(2)
                    .map(|x| self.read_operand_other(((op_types >> x) & 0b11) as u8))
                    .collect()
            }
            Instruction::Variable(_) | Instruction::Extended(_) => {
                let op_types = self.cursor.read_byte();
                operands = (0..=6)
                    .rev()
                    .step_by(2)
                    .map(|x| self.read_operand_other((op_types >> x) & 0b11))
                    .collect();
            }
            Instruction::Long(_) => {
                for x in 6..=5 {
                    operands.push(self.read_operand_long((op >> x) & 0b1));
                }
            }
        }
    }
}
