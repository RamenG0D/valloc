use std::{error::Error, fmt::{Display, Formatter}};


#[derive(Debug)]
pub enum AllocError {
    OOM,
    InvalidPointer,
    ChunkNotFound,
}

impl Display for AllocError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            AllocError::OOM => write!(f, "Out of memory"),
            AllocError::InvalidPointer => write!(f, "Invalid pointer"),
            AllocError::ChunkNotFound => write!(f, "Chunk not found"),
        }
    }
}

impl Error for AllocError {}
