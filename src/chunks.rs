use std::io::Read;

use bytes::{Buf, Bytes};
use crab_nbt::{Nbt, NbtTag};
use enum_utils::TryFromRepr;
use flate2::bufread::{GzDecoder, ZlibDecoder};

use crate::error::ChunkLoadError;
use crate::error::ChunkLoadError::*;
use crate::chunks::sections::ChunkSection;

pub mod sections;
pub mod iterators;
mod utils;

#[derive(Debug, TryFromRepr)]
#[repr(u8)]
pub enum CompressionFormat {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
    Lz4 = 4,
    // There could be Custom = 127 here, but we couldn't support it anyway
}

pub enum ChunkStatus {
    Empty,
    StructureStarts, StructureReferences,
    Biomes,
    Noise,
    Surface,
    Carvers, LiquidCarvers,
    Features,
    Light, InitializeLight,
    Spawn,
    Full
}
impl TryFrom<&str> for ChunkStatus {
    type Error = ChunkLoadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "minecraft:empty" => ChunkStatus::Empty,
            "minecraft:structure_starts" => ChunkStatus::StructureStarts,
            "minecraft:structure_references" => ChunkStatus::StructureReferences,
            "minecraft:biomes" => ChunkStatus::Biomes,
            "minecraft:noise" => ChunkStatus::Noise,
            "minecraft:surface" => ChunkStatus::Surface,
            "minecraft:carvers" => ChunkStatus::Carvers,
            "minecraft:liquid_carvers" => ChunkStatus::LiquidCarvers,
            "minecraft:features" => ChunkStatus::Features,
            "minecraft:light" => ChunkStatus::Light,
            "minecraft:initialize_light" => ChunkStatus::InitializeLight,
            "minecraft:spawn" => ChunkStatus::Spawn,
            "minecraft:full" => ChunkStatus::Full,
            _ => return Err(MalformedChunk("Chunk has unexpected status"))
        })
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub data: Nbt
}
impl Chunk {
    pub(crate) fn read(buf: &[u8]) -> Result<Self, ChunkLoadError> {
        let size = buf.get(0..4)
            .ok_or(ChunkLoadError::MalformedChunk("Header is too short, should be 4 bytes"))?
            .try_into()
            .map(|b| u32::from_be_bytes(b) - 1)
            .unwrap(); // converting this &[u8] into a [u8;4] will never fail
        let compression_format = buf.get(4)
            .ok_or(ChunkLoadError::MalformedChunk("Chunk is too short for header (len<5)"))
            .map(|x| CompressionFormat::try_from(*x))?
            .map_err(|_| ChunkLoadError::UnknownCompressionFormat(buf[4]))?;

        let mut decompressed: Box<dyn Buf>;
        {
            let compressed = &buf[5..((size+5) as usize)];
            decompressed = match compression_format {
                // FIXME: Gzip & Zlib decoders constantly reallocate vec
                //  (It appears the implementation is slightly stupid)
                CompressionFormat::Gzip => {
                    let mut vec = Vec::new();
                    GzDecoder::new(compressed).read_to_end(&mut vec)?;
                    Box::new(Bytes::from(vec))
                },
                CompressionFormat::Zlib => {
                    let mut vec = Vec::new();
                    ZlibDecoder::new(compressed).read_to_end(&mut vec)?;
                    Box::new(Bytes::from(vec))
                },
                CompressionFormat::Lz4 => {
                    let mut vec = Vec::new();
                    lz4::Decoder::new(compressed)?.read_to_end(&mut vec)?;
                    Box::new(Bytes::from(vec))
                },
                CompressionFormat::Uncompressed => Box::new(compressed)
            };
        }
        let nbt = Nbt::read(&mut decompressed)?;
        Ok(Chunk {
            data: nbt
        })
    }

    pub fn get_subchunks(&self) -> Result<SectionIterator<'_>, ChunkLoadError> {
        self.get_sections().map(|sections| SectionIterator {
            section_tags: sections.iter()
        })
    }

    pub fn get_subchunk(&self, index: usize) -> Result<ChunkSection<'_>, ChunkLoadError> {
        parse_chunk(self.get_sections()?.get(index).ok_or(MissingSection)?)
    }

    fn get_sections(&self) -> Result<&Vec<NbtTag>, ChunkLoadError> {
        self.data.get_list("sections")
            .ok_or(MalformedChunk("Chunk has no sections list object"))
    }
}

fn parse_chunk<'a>(tag: &'a NbtTag) -> Result<ChunkSection<'a>, ChunkLoadError> {
    tag.extract_compound()
        .ok_or(MalformedChunk("Chunk section is not a compound"))
        .and_then(|compound| ChunkSection::new(compound))
}

pub struct SectionIterator<'a> {
    section_tags: std::slice::Iter<'a, NbtTag>
}
impl<'a> Iterator for SectionIterator<'a> {
    type Item = Result<ChunkSection<'a>, ChunkLoadError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.section_tags.next().map(|tag| parse_chunk(tag))
    }
}