mod address;
mod alphabet;
pub mod error;
pub mod input_code;
mod instruction;
mod memory;
mod property;
mod stack;
pub mod state;
pub use input_code::InputCode;

pub type Result<T> = std::result::Result<T, error::GameError>;
