pub enum Result {
    Continue,
    Return(u16),
    Quit,
    Invoke {
        address: usize,
        store_to: Option<u8>,
        arguments: Option<Vec<u16>>,
    },
}
