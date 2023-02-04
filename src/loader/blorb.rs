use std::collections::HashMap;

use crate::game::Result;
use crate::game::error::GameError;


pub enum IndexKind {
    Picture,
    Sound,
    Data,
    Executable,
}

impl IndexKind {
    pub fn from_index_name(name: &str) -> Option<IndexKind> {
        match name {
            "Pict" => Some(IndexKind::Picture),
            "Snd " => Some(IndexKind::Sound),
            "Exec" => Some(IndexKind::Executable),
            "Data" => Some(IndexKind::Data),
            _ => None,
        }
    }
}

enum ExectuableSystem {
    ZCode,
    Glulx,
    TADS2,
    TADS3,
    Hugo,
    Alan,
    Adrift,
    Level9,
    AGT,
    MagneticScrolls,
    AdvSys,
    Native,
    Other(String),
}

enum PictureFormat {
    Png,
    Jpeg,
    Placeholder,
}

enum SoundFormat {
    Ogg,
    Aiff,
    Mod,
    Song,
}

enum ChunkKind {
    Picture { format: PictureFormat },
    Sound { format: SoundFormat },
    Data,
    Executable { system: ExecutableSystem },
}

pub struct Chunk<'a> {
    data: &'a [u8],
    kind: ChunkKind,
}

pub struct BlorbLoader {
    data: Vec<u8>,
}

pub struct ChunkIter<'a> {
    loader: &'a BlorbLoader,
    from: usize,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Chunk;
    fn next(&mut self) -> Option<Self::Item> {
        if from >= self.loader.data.len() {
            return None
        }
    }
}

fn invalid_file_error() -> GameError {
    GameError::invalid_file().detail("Not a blorb file.")
}

trait IifHelpers {
    /// Require th 
    fn try_read(data: &[u8], range: I) -> Result<&<I as SliceIndex<[T]>>::Output> where I: SliceIndex<[T]>  {
}

fn require_text(data: &[u8], text: &str) -> Result<()> {
    if data[..text.len()] != text.as_bytes() {
        Err(GameError::invalid_file().detail("Not a blorb file."))
    } else {
        Ok(())
    }
}

fn try_read(data: &[u8], range: I) -> Result<&<I as SliceIndex<[T]>>::Output> where I: SliceIndex<[T]>  {
    data.get(range).ok_or_else(invalid_file_error) 
}

impl BlorbLoader {
    pub fn new<D: Into<Vec<u8>>(data: D) -> Result<BlorbFile> {
        let data: Vec<u8> = data.into();
        require_text(try_read(&data, 0..4)?, &"FORM")?;

        let data_len = u32::from_be_bytes(&[4..8]);
        if data_len != data.len() - 8 {
            return Err(invalid_file_error())
        }

        require_text(&data[8..12], &"IFRS")?;

        let index = Self::read_index(&[12..])?;

        BlorbFile {
            data,
        }
    }


    fn chunks(&'a self) -> ChunkIter {
    }
}
