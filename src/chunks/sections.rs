use std::fmt::Display;

use crab_nbt::{NbtCompound, NbtTag};

use crate::chunks::iterators::BlockIter;
use crate::error::ChunkLoadError;
use crate::error::ChunkLoadError::*;

static EMPTY_VEC_I64: Vec<i64> = Vec::new();
static EMPTY_VEC_BLOCK_PROPERTIES: Vec<(String, NbtTag)> = Vec::new();

#[derive(Debug)]
pub struct ChunkSection<'a> {
    pub y: i8,
    pub blocks: SectionBlocks<'a>
}
impl<'a> ChunkSection<'a> {
    pub(crate) fn new(compound: &'a NbtCompound) -> Result<Self, ChunkLoadError> {
        Ok(Self {
            y: compound.get_byte("Y").ok_or(MalformedChunk("Section missing Y value"))?,
            blocks: SectionBlocks::new(compound.get_compound("block_states")
                .ok_or(EmptySection)?)?
        })
    }
}

#[derive(Debug)]
pub struct SectionBlocks<'a> {
    pub(super) palette: Vec<BlockState<'a>>,
    pub(super) data: &'a Vec<i64>
}
impl<'a> SectionBlocks<'a> {
    fn new(compound: &'a NbtCompound) -> Result<Self, ChunkLoadError> {
        let palette = match compound.get_list("palette") {
            None => Vec::new(),
            Some(palette_raw) => {
                let mut palette = Vec::with_capacity(palette_raw.len());
                for raw in palette_raw {
                    let compound = raw.extract_compound()
                        .ok_or(MalformedChunk("Palette entry is not a compound"))?;
                    palette.push(BlockState::new(compound)
                        .ok_or(MalformedChunk("Palette entry is not a valid block state"))?)
                }
                palette
            }
        };
        Ok(Self {
            palette: palette,
            data: compound.get_long_array("data")
                .unwrap_or(&EMPTY_VEC_I64)
        })
    }

    pub fn get_palette(&self) -> &Vec<BlockState<'a>> {
        &self.palette
    }
}
impl<'a> IntoIterator for &'a SectionBlocks<'a> {
    type Item = &'a BlockState<'a>;
    type IntoIter = BlockIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BlockIter::new(self)
    }
}

#[derive(Debug)]
pub struct BlockState<'a> {
    /// The namespaced id of the Block
    pub name: &'a String,
    /// A list of block state properties. Empty, if no properties exist
    pub properties: &'a Vec<(String, NbtTag)>
}
impl<'a> BlockState<'a> {
    fn new(compound: &'a NbtCompound) -> Option<Self> {
        let name = compound.get_string("Name")?;
        let properties = compound.get_compound("Properties")
            .map(|x| &x.child_tags)
            .unwrap_or(&EMPTY_VEC_BLOCK_PROPERTIES);
        Some(BlockState { name: name, properties: properties })
    }
}
impl<'a> Display for BlockState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.properties.is_empty() {
            write!(f, "[")?;
            for (name, value) in self.properties {
                write!(f, "{}={:?}", name, value)?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}