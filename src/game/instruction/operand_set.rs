use crate::game::error::GameError;
use itertools::Itertools;
use std::fmt::{self, Debug, Display, Formatter};

use super::Operand;

pub struct OperandSet {
    index: usize,
    pub set: Vec<Operand>,
}

impl OperandSet {
    pub fn new(set: Vec<Operand>) -> OperandSet {
        OperandSet { index: 0, set }
    }

    pub fn pull(&mut self) -> Result<Operand, GameError> {
        self.next()
            .ok_or_else(|| GameError::InvalidOperation("Instruction has too few operands".into()))
    }
}

impl Iterator for OperandSet {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.set.len() {
            None
        } else {
            let result = Some(self.set[self.index]);
            self.index += 1;
            result
        }
    }
}

impl Display for OperandSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.set
                .iter()
                .filter(|x| !matches!(x, Operand::Omitted))
                .join(",")
        )
    }
}

impl Debug for OperandSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
