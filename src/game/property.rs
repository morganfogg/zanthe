use anyhow::Result;

use crate::game::error::GameError;

pub struct Property {
    pub number: u16,
    pub address: u16,
    pub data_address: u16,
    pub data: Vec<u8>,
}

impl Property {
    pub fn data_to_u16(&self) -> Result<u16> {
        match self.data.len() {
            1 => Ok(self.data[0] as u16),
            2 => Ok(((self.data[0] as u16) << 8) + self.data[1] as u16),
            _ => {
                Err(GameError::InvalidOperation("Property data too long to convert".into()).into())
            }
        }
    }
}
