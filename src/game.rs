mod address;
mod alphabet;
pub mod input_code;
mod instruction;
mod memory;
mod property;
mod stack;
pub mod state;
pub mod error;

pub use input_code::InputCode;


pub type Result<T> = std::result::Result<T, error::GameError>;
