use crate::chunks::sections::BlockState;

pub(crate) fn unpack_value(value: i64, offset: u8, bits_per_block: u8) -> usize {
    let mask = ((1u64 << bits_per_block) - 1) << offset;
    // "as" performs raw conversions. Notice! Using u64 is very necessary here.
    //  right-shifting on signed types preserves the sign in Rust (c.f. Arithmetic Shift)
    //  this would cause massive problems. 
    //  Converting to u64 beforhand ensures logical shifting is used instead.
    return ((value as u64 & mask) >> offset) as usize;
}

pub(crate) fn calculate_bits_per_block<'a>(palette: &Vec<BlockState<'a>>) -> u8 {
    let x = palette.len() - 1;
    // TODO: Is this really the fastest way to do it?
    ((x.checked_ilog2().unwrap_or(0) + 1) as u8).max(4)
}