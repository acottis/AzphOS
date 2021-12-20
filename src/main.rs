/// Custom Result type to take advantage of our custom Error messaging
///  
type Result<T> = std::result::Result<T, self::Error>;

/// Custom Error Enum for better Error reporting
/// 
#[derive(Debug)]
enum Error{
    PENotFound(std::io::Error),
    CargoMissing(std::io::Error),
    CommandDidNotComplete,
    CargoBuildFailed(String),
    CantConvertToUtf(std::string::FromUtf8Error),
    Consume(std::io::Error),
    BadSigniture,
    Seek(std::io::Error)
}

/// Main program loop
/// 
fn main(){
    // This function compiles the bootloader that we will use as a stage0
    build_bootloader().expect("Failed to build bootloader");

    // Parse the bootloader
    // TODO
    Pe::parse("bootloader/target/i586-pc-windows-msvc/release/bootloader.exe").expect("Failed to parse PE");
}
/// This function comiples the bootloader in the subfolder and returns an error if it fails
///
fn build_bootloader() -> Result<()>{
    use std::process::Command;

    let res = Command::new("cargo").args(["build", "--release"]).current_dir("bootloader").output().map_err(Error::CargoMissing)?;

    match &res.status.code(){
        Some(0) => {
            println!("Bootloader: Cargo Build Sucessful");
            Ok(())
        },
        Some(_) => {
            let stderr = String::from_utf8(res.stderr).map_err(Error::CantConvertToUtf)?;
            return Err(Error::CargoBuildFailed(stderr))
        },
        None => return Err(Error::CommandDidNotComplete),
    }
}

/// Unsure
/// 
struct Pe;

impl Pe{
    /// Takes the ref to a path and parses the PE header
    /// 
    fn parse(path: impl AsRef<std::path::Path>) -> Result<()>{
        use std::io::Read;
        use std::io::{Seek, SeekFrom};
        const POINTER_TO_PE_HEADER_OFFSET: u64 = 0x3C;
        const PE_SIGNITURE: [u8; 2] = *b"MZ";


        // Open file and get reader over its buffer
        let pe = std::fs::File::open(&path).map_err(Error::PENotFound)?;
        let mut reader = std::io::BufReader::new(&pe);

        // Check for magic
        if consume!(reader, u16, "Signiture") != PE_SIGNITURE {
            return Err(Error::BadSigniture);
        }

        // Skip ahead to PE Header Pointer
        reader.seek(SeekFrom::Start(POINTER_TO_PE_HEADER_OFFSET)).map_err(Error::Seek)?;
        
        let header_pointer = consume!(reader, u32, "Pointer To PE Header");
        println!("Header is at {:X?}", header_pointer);
 

        Ok(())
    }

}


/// Helper for reading through file Buffer
/// Takes an expression (BufReader), A Type Big enough for the field we are passing (u32), A String describing the field
#[macro_export]
macro_rules! consume {
    ($reader:expr, $ty:ty, $field:expr) => {{
        let mut buf = [0u8; std::mem::size_of::<$ty>()];
        $reader.read_exact(&mut buf).map_err(Error::Consume)?;
        buf
    }}
}