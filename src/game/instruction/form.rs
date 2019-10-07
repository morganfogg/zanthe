/// Represents the different ways an instruction can be encoded in memory.
#[derive(Debug, PartialEq)]
pub enum Form {
    Long,
    Short,
    Extended,
    Variable,
}
