use std::io::Read;

use bytes::{Buf, Bytes};
use crab_nbt::Nbt;
use enum_utils::TryFromRepr;
use flate2::bufread::{GzDecoder, ZlibDecoder};

use crate::error::ChunkLoadError;
use crate::error::ChunkLoadError::*;
use crate::chunks::sections::ChunkSection;

pub mod sections;
pub mod iterators;

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
        let size: u32 = u32::from_be_bytes(buf[0..4].try_into().unwrap()) - 1;
        let compression_format = CompressionFormat::try_from(buf[4])
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

    pub fn get_subchunk<'a>(&'a self, index: usize) -> Result<ChunkSection<'a>, ChunkLoadError> {
        let section = self.data.get_list("sections")
            .ok_or(MalformedChunk("Chunk has no sections list object"))?
            .get(index).ok_or(MissingSection)?
            .extract_compound().ok_or(MalformedChunk("Chunk section is not a compound"))?;
        ChunkSection::new(section)
    }
}
