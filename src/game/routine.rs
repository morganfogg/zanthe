use crate::game::memory::Memory;

pub struct Routine<'a> {
    counter: usize,
    stack: Vec<u8>,
    variables: Vec<u8>,
    version: u8,
    memory: &'a mut Memory,
}

impl<'a> Routine<'a> {
    pub fn new(memory: &mut Memory, counter: usize) -> Routine {
        Routine {
            stack: Vec::new(),
            variables: Vec::new(),
            counter,
            version: memory.version(),
            memory,
        }
    }
    pub fn invoke(&mut self) {}
}
