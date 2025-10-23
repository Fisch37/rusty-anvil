use std::io::Cursor;

use rusty_anvil::RegionFileReader;

#[test]
fn can_load() {
    let mut reader = RegionFileReader::create(
        Cursor::new(&include_bytes!("data/superflat-colored.mca")[..])).unwrap();
    let chunk = reader.get_chunk(0, 31).unwrap();

    let subchunk = chunk.get_subchunk(1).unwrap();
    for ([x,y,z], block) in (&subchunk.blocks).into_iter().with_coordinates() {
        let b2 = subchunk.blocks.get_block(x, y, z);
        assert_eq!(block, b2, "{x},{y},{z} failed to match with iterator. {block}!={b2}")
    }
}