use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IffReadError {
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("Format error: {0}")]
    FormatError(String),
}

type Result<T> = std::result::Result<T, IffReadError>;

#[derive(Debug, Clone)]
pub struct FormChunk {
    kind: [u8; 4],
    chunks: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub struct DataChunk {
    kind: [u8; 4],
    data: Vec<u8>,
}

// TODO: Add LIST and CAT.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Chunk {
    Form(FormChunk),
    Data(DataChunk),
}

pub struct IffReader<F: Read + Seek> {
    reader: F,
}

impl<F: Read + Seek> IffReader<F> {
    pub fn new(reader: F) -> IffReader<F> {
        Self { reader }
    }

    fn read_chunk(&mut self) -> Result<Chunk> {
        let mut word = [0u8; 4];
        let mut len = [0u8; 4];

        self.reader.read_exact(&mut word)?;
        self.reader.read_exact(&mut len)?;
        let len = u32::from_be_bytes(len);

        let result = match &word {
            b"FORM" => {
                let mut kind = [0u8; 4];
                self.reader.read_exact(&mut kind)?;
                let mut chunks = Vec::new();
                while self.reader.stream_position()? < len as u64 {
                    chunks.push(self.read_chunk()?);
                }
                Ok(Chunk::Form(FormChunk { kind, chunks }))
            }
            b"LIST" | b"CAT " => {
                todo!();
            }
            _ => {
                if len < 4 {
                    return Err(IffReadError::FormatError("Invalid length specifier".into()));
                }
                let mut data = vec![0u8; len as usize - 4];
                let mut kind = [0u8; 4];
                self.reader.read_exact(&mut kind)?;
                self.reader.read_exact(&mut data)?;
                Ok(Chunk::Data(DataChunk { kind, data }))
            }
        };
        if self.reader.stream_position()? % 2 == 1 {
            self.reader.seek(SeekFrom::Current(1))?;
        }
        result
    }

    pub fn load(&mut self) -> Result<Chunk> {
        let mut word = [0u8; 4];
        self.reader.read_exact(&mut word)?;
        if !matches!(&word, b"FORM" | b"LIST" | b"CAT ") {
            return Err(IffReadError::FormatError("Not an IFF file".into()));
        }

        self.reader.rewind()?;

        let chunk = self.read_chunk()?;

        let pos = self.reader.stream_position()?;
        let end = self.reader.seek(SeekFrom::End(0))?;

        if pos != end {
            return Err(IffReadError::FormatError("Trailing data".into()));
        }

        Ok(chunk)
    }
}
