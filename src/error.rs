use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ChunkLoadError {
    MissingSection,
    EmptySection,
    MalformedChunk(&'static str),
    ChunkDoesNotExist,
    IOError(std::io::Error),
    UnknownCompressionFormat(u8),
    MalformedNbt(crab_nbt::error::Error),
}
impl From<std::io::Error> for ChunkLoadError {
    fn from(value: std::io::Error) -> Self {
        ChunkLoadError::IOError(value)
    }
}
impl From<crab_nbt::error::Error> for ChunkLoadError {
    fn from(value: crab_nbt::error::Error) -> Self {
        ChunkLoadError::MalformedNbt(value)
    }
}
impl Display for ChunkLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for ChunkLoadError { }