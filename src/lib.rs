use std::io::{Read, Seek, SeekFrom};

use crate::{chunks::Chunk, error::ChunkLoadError, metadata::{ChunkTimestamp, LocationTable, TimestampTable}};

pub mod error;
pub mod chunks;
pub mod metadata;

const TABLE_SIZE: usize = 1024;
const ENTRY_SIZE: usize = 4;
const LOCATION_SIZE_FACTOR: usize = 4096;

pub struct RegionFileReader<R: Read + Seek> {
    reader: R,
    location_table: LocationTable,
    timestamp_table: TimestampTable
}
impl<R: Read + Seek> RegionFileReader<R> {
    pub fn create(mut reader: R) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(0))?;
        Ok(RegionFileReader {
            location_table: LocationTable::read(&mut reader)?,
            timestamp_table: TimestampTable::read(&mut reader)?,
            reader: reader // order is weird because of mutable borrows above
        })
    }

    pub fn get_timestamps(&self) -> &TimestampTable {
        &self.timestamp_table
    }

    pub fn get_timestamp(&self, chunk_x: u8, chunk_z: u8) -> Option<ChunkTimestamp> {
        self.get_timestamps().as_ref()
            .get(get_chunk_index(chunk_x, chunk_z)).copied()
    }

    pub fn get_chunk(&mut self, chunk_x: u8, chunk_z: u8) -> Result<Chunk, ChunkLoadError> {
        let location = &self.location_table[get_chunk_index(chunk_x, chunk_z)];
        
        let (seek, size) = location.to_offset_form();
        if size == 0 {
            return Err(ChunkLoadError::ChunkDoesNotExist)
        }
        self.reader.seek(SeekFrom::Start(seek))?;
        let mut buf = vec![0u8; size];
        self.reader.read_exact(&mut buf)?;

        Ok(Chunk::read(&buf)?)
    }
}

fn get_chunk_index(chunk_x: u8, chunk_z: u8) -> usize {
    chunk_z as usize * 32 + chunk_x as usize
}
