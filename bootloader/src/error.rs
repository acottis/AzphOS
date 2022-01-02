pub type Result<T> = core::result::Result<T, self::Error>;

#[derive(Debug)]
pub enum Error{
    UnsupportedNIC((u16,u16)),
    NoNICFound,
}