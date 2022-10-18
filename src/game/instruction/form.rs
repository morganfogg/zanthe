/// Represents the different ways an instruction can be encoded in memory.
#[derive(Debug, PartialEq, Eq)]
pub enum Form {
    Long,
    Short,
    Extended,
    Variable,
}
