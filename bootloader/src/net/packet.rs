#[derive(Clone, Copy)]
pub struct Packet{}

impl Packet{
    pub fn parse(buf: &[u8], len: u16) {}

    /// Takes a buffer and fills in the data and then returns the length
    pub fn new(buf: &mut [u8]) -> usize {
        0
    }
}