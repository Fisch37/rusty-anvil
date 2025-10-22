use crate::chunks::sections::{BlockState, SectionBlocks};

pub struct BlockIter<'a> {
    section: &'a SectionBlocks<'a>,
    values_returned: u16,
    bits_per_block: u8,
    array_index: usize,
    offset: u8
}
impl<'a> BlockIter<'a> {
    pub(super) fn new(section: &'a SectionBlocks<'a>) -> Self {
        BlockIter {
            section: section,
            values_returned: 0,
            bits_per_block: {
                let x = section.palette.len() - 1;
                // TODO: Is this really the fastest way to do it?
                ((x.checked_ilog2().unwrap_or(0) + 1) as u8).max(4)
            },
            array_index: 0,
            offset: 0
        }
    }

    pub fn with_coordinates(self) -> BlockCoordinatesIter<'a> {
        BlockCoordinatesIter { iterator: self, i: 0 }
    }
}
impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a BlockState<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.values_returned += 1;
        let block_index;
        if self.section.data.len() == 0 {
            block_index = if self.values_returned > 4096 { None } else { Some(0) };
        } else {
            block_index = self.section.data.get(self.array_index)
                .map(|value| unpack_value(*value, self.offset, self.bits_per_block));
            self.offset += self.bits_per_block;
            if u32::from(self.offset) >= i64::BITS {
                self.offset = 0;
                self.array_index += 1;
            }
        }
        block_index.map(|x| &self.section.palette[x])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO: Validate this against the iterator state
        (4096, Some(4096))
    }
}

fn unpack_value(value: i64, offset: u8, bits_per_block: u8) -> usize {
    let mask = ((1u64 << bits_per_block) - 1) << offset;
    // "as" performs raw conversions. Notice! Using u64 is very necessary here.
    //  right-shifting on signed types preserves the sign in Rust (c.f. Arithmetic Shift)
    //  this would cause massive problems. 
    //  Converting to u64 beforhand ensures logical shifting is used instead.
    return ((value as u64 & mask) >> offset) as usize;
}

pub struct BlockCoordinatesIter<'a> {
    iterator: BlockIter<'a>,
    i: u16
}
impl<'a> Iterator for BlockCoordinatesIter<'a> {
    type Item = ([u8; 3], &'a BlockState<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.iterator.next().map(|value| {
            let x = (self.i % 16) as u8;

            let j = self.i / 16;
            let z = (j % 16) as u8;
            let y = (j / 16) as u8;
            ([x, y, z], value)
        });
        self.i += 1;
        value
    }
}