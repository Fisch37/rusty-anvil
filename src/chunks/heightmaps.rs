use crate::chunks::utils::{get_index_offset_form, unpack_value};


const BITS_PER_VALUE: u8 = 9;
const HEIGHTMAP_LENGTH: u16 = 16 * 16;

pub enum HeightmapType {
    MotionBlocking,
    MotionBlockingNoLeaves,
    OceanFloor,
    WorldSurface
}
impl HeightmapType {
    pub fn get_identifier(&self) -> &'static str {
        match self {
            HeightmapType::MotionBlocking => "MOTION_BLOCKING",
            HeightmapType::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
            HeightmapType::OceanFloor => "OCEAN_FLOOR",
            HeightmapType::WorldSurface => "WORLD_SURFACE",
        }
    }
}

pub struct Heightmap<'a> {
    data: &'a Vec<i64>
}
impl<'a> Heightmap<'a> {
    pub fn new(data: &'a Vec<i64>) -> Self {
        Heightmap { data: data }
    }

    /// Returns the distance from the world floor at the relative xz position in a chunk.
    /// Note that the resulting value is unsigned and thus requires subtracting 64 to get a y-value.
    pub fn get_at(&self, x: u8, z: u8) -> u16 {
        let i = x as u16 + (z as u16)*16;
        let (index, offset) = get_index_offset_form(i, BITS_PER_VALUE, i64::BITS as u8);
        unpack_value::<u16>(self.data[index], offset, BITS_PER_VALUE)
    }
}
impl<'a> IntoIterator for &'a Heightmap<'a> {
    type Item = u16;

    type IntoIter = HeightmapIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeightmapIterator {
            data: self.data, i: 0,
            index: 0, offset: 0
        }
    }
}

pub struct HeightmapIterator<'a> {
    data: &'a Vec<i64>,
    i: u16,
    index: usize,
    offset: u8
}
impl<'a> HeightmapIterator<'a> {
    pub fn with_coordinates(self) -> impl Iterator<Item = ([u8;2], u16)> {
        self.enumerate().map(|(i, height)| ([(i % 16) as u8, (i / 16) as u8], height))
    }
}
impl<'a> Iterator for HeightmapIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= HEIGHTMAP_LENGTH {
            return None;
        }
        let result = self.data.get(self.index)
            .map(|x| unpack_value::<u16>(*x, self.offset, BITS_PER_VALUE));
        self.offset += BITS_PER_VALUE;
        if self.offset >= (i64::BITS as u8 - BITS_PER_VALUE) {
            self.offset = 0;
            self.index += 1;
        }
        self.i += 1;
        result
    }
}