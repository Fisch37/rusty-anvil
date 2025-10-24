use std::{io::Read, ops::{Deref, Index}};

use crate::{ENTRY_SIZE, LOCATION_SIZE_FACTOR, TABLE_SIZE};

#[derive(Debug)]
pub(crate) struct LocationTableEntry {
    // position within the file (starting at 0) in 4096 bytes
    position: u32,
    // size in 4096 bytes
    size: u8
}
impl LocationTableEntry {
    fn from_bytes(bytes: &[u8]) -> Self {
        LocationTableEntry {
            position: u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]),
            size: bytes[3]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.position == 0 && self.size == 0
    }

    /// Converts this entry into the offset form.
    /// This means the first value is the offset within the file in bytes,
    /// and the second is the size of the chunk in bytes (rounded up to the nearest factor of 4096)
    pub(crate) fn to_offset_form(&self) -> (u64, usize) {
        (
            self.position as u64 * LOCATION_SIZE_FACTOR as u64,
            self.size as usize * LOCATION_SIZE_FACTOR
        )
    }
}

pub(crate) struct LocationTable {
    internal: Vec<LocationTableEntry>
}
impl LocationTable {
    pub(crate) fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; TABLE_SIZE * ENTRY_SIZE];
        reader.read_exact(&mut buf)?;

        // Not all that happy with using a Vec here (array could work to, because constant-tiem size)
        // but I can't really work safely with uninitialised arrays in Rust :(
        let mut location_table = Vec::with_capacity(TABLE_SIZE);
        for i in 0..TABLE_SIZE {
            let pos = i * 4;
            location_table.push(LocationTableEntry::from_bytes(&buf[pos..pos+4]))
        }
        Ok(LocationTable { internal: location_table })
    }
}
impl Index<usize> for LocationTable {
    type Output = LocationTableEntry;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.internal[index]
    }
}

pub type ChunkTimestamp = i32;
pub struct TimestampTable {
    internal: Vec<ChunkTimestamp>
}
impl TimestampTable {
    pub(crate) fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; TABLE_SIZE * ENTRY_SIZE];
        reader.read_exact(&mut buf)?;

        let mut timestamp_table = Vec::with_capacity(TABLE_SIZE);
        for i in 0..TABLE_SIZE {
            let pos = i * 4;
            timestamp_table.push(ChunkTimestamp::from_be_bytes(buf[pos..pos+4].try_into()
                .expect("Slice has unexpected length. This should never happen")));
        }
        Ok(TimestampTable { internal: timestamp_table })
    }
}
impl Index<usize> for TimestampTable {
    type Output = ChunkTimestamp;

    fn index(&self, index: usize) -> &Self::Output {
        &self.internal[index]
    }
}
impl From<TimestampTable> for Vec<ChunkTimestamp> {
    fn from(value: TimestampTable) -> Self {
        value.internal
    }
}
impl AsRef<Vec<ChunkTimestamp>> for TimestampTable {
    fn as_ref(&self) -> &Vec<ChunkTimestamp> {
        self.deref()
    }
}
impl Deref for TimestampTable {
    type Target = Vec<ChunkTimestamp>;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}