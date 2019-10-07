/// The result of an instruction
pub enum Result {
    /// Continue executing the current routine.
    Continue,
    /// Return from the current routine with the given value.
    Return(u16),
    /// Quit the game.
    Quit,
    /// Call a new routine.
    Invoke {
        address: usize,
        store_to: Option<u8>,
        arguments: Option<Vec<u16>>,
    },
}
