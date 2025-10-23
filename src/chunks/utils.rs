use std::fmt::Debug;
use crate::chunks::sections::BlockState;

pub(crate) fn unpack_value<I>(value: i64, offset: u8, bits_per_value: u8) -> I 
    where I: TryFrom<u64, Error: Debug>
{
    let mask = ((1u64 << bits_per_value) - 1) << offset;
    // "as" performs raw conversions. Notice! Using u64 is very necessary here.
    //  right-shifting on signed types preserves the sign in Rust (c.f. Arithmetic Shift)
    //  this would cause massive problems. 
    //  Converting to u64 beforhand ensures logical shifting is used instead.
    return ((value as u64 & mask) >> offset).try_into().unwrap();
}

pub(crate) fn calculate_bits_per_block<'a>(palette: &Vec<BlockState<'a>>) -> u8 {
    let x = palette.len() - 1;
    // TODO: Is this really the fastest way to do it?
    ((x.checked_ilog2().unwrap_or(0) + 1) as u8).max(4)
}

pub(crate) fn get_index_offset_form(i: u16, bits_per_value: u8, packed_bits: u8) -> (usize, u8) {
    let values_per_long = packed_bits / bits_per_value;
    let index = i as usize / values_per_long as usize;
    let offset = ((i * bits_per_value as u16) % packed_bits as u16) as u8;
    (index, offset)
}