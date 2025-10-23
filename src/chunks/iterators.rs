use crate::chunks::{sections::{BlockState, SectionBlocks}, utils::{calculate_bits_per_block, unpack_value}};

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
            bits_per_block: calculate_bits_per_block(&section.palette),
            array_index: 0,
            offset: 0
        }
    }

    pub fn with_coordinates(self) -> impl Iterator<Item = ([u8;3], &'a BlockState<'a>)> {
        self.enumerate().map(|(i, block)| {
            let x = (i % 16) as u8;

            let j = i / 16;
            let z = (j % 16) as u8;
            let y = (j / 16) as u8;
            ([x, y, z], block)
        })
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
