/// Custom Result type to take advantage of our custom Error messaging
///  
type Result<T> = std::result::Result<T, self::Error>;

/// Custom Error Enum for better Error reporting
/// 
#[derive(Debug)]
enum Error{
    CouldNotReadSectionData,
    PENotFound(std::io::Error),
    CargoMissing(std::io::Error),
    NasmMissing(std::io::Error),
    CommandDidNotComplete,
    CargoBuildFailed(String),
    NasmBuildFailed(String),
    CantConvertToUtf(std::string::FromUtf8Error),
    Consume(std::io::Error),
    BadDOSSigniture,
    BadPESigniture,
    Seek(std::io::Error),
    UnsupportedMachineType(u16),
    UnsupportedOptionalHeaderMagic(u16),
    CantCreateBinary(std::io::Error),
}

/// Main program loop
/// 
fn main(){
    const FLATTENED_IMAGE_PATH: &'static str = "bootloader/build/bootloader.flat";
    
    // This function compiles the bootloader that we will use as a stage0
    build_bootloader().expect("Failed to build bootloader");

    // Parse the bootloader
    let pe = Pe::parse("bootloader/target/i586-pc-windows-msvc/release/bootloader.exe").expect("Failed to parse PE");
    //println!("{:#X?}", pe.sections);

    let flattened_bytes = pe.flatten().expect("Could not flatten PE");
    //println!("{:X?}", program_bytes);
    println!("Flattened PE is {:#X} bytes", &flattened_bytes.len());
    
    write_flattened_image(&flattened_bytes, FLATTENED_IMAGE_PATH).expect("Could not write image to disk");
    println!("Entry at: {:#X}", pe.entry_point);

    build_asm(pe.entry_point).expect("Cannot assemble stage0.asm");
    println!("PE Written to: {}", FLATTENED_IMAGE_PATH)


}
/// This functions writes the flattened PE to disk
/// 
fn write_flattened_image(bytes: &[u8], path :&str) -> Result<()>{
    use std::io::Write;
    let mut output = std::fs::File::create(path).map_err(Error::CantCreateBinary)?;
    output.write(&bytes).map_err(Error::CantCreateBinary)?;
    Ok(())
}
/// This function compiles the assembly code with the entry point found in the PE
/// 
fn build_asm(entry: u32) -> Result<()>{
    use std::process::Command;

    let res = Command::new("nasm").args(
        ["bootloader/asm/stage0.asm", 
        "-f", 
        "bin",
        &format!("-Dentry_point={:#X}", entry),
        "-o", 
        "bootloader/build/stage0.bin"]
        ).output().map_err(Error::NasmMissing)?;

    match &res.status.code(){
        Some(0) => {
            println!("Bootloader: Nasm sucess");
            Ok(())
        },
        Some(_) => {
            let stderr = String::from_utf8(res.stderr).map_err(Error::CantConvertToUtf)?;
            return Err(Error::NasmBuildFailed(stderr))
        },
        None => return Err(Error::CommandDidNotComplete),
    }
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
struct Pe{
    entry_point: u32,
    sections: Vec<Section>,
    bytes: Vec<u8>
}

impl Pe{
    /// Takes the ref to a path and parses the PE header
    /// https://docs.microsoft.com/en-gb/windows/win32/debug/pe-format?redirectedfrom=MSDN#ms-dos-stub-image-only
    /// 
    fn parse(path: impl AsRef<std::path::Path>) -> Result<Self>{
        use std::io::Read;
        use std::io::{Seek, SeekFrom};
        const POINTER_TO_PE_HEADER_OFFSET: u64 = 0x3C;
        const DOS_SIGNITURE: u16 = 0x5A4D; // b"MZ"
        const PE_SIGNITURE: u32 = 0x00004550; // b"PE\0\0"
        const COFF_HEADER_SIZE: u32 = 0x18;

        // Open file and get reader over its buffer
        let pe = std::fs::File::open(&path).map_err(Error::PENotFound)?;
        let mut reader = std::io::BufReader::new(&pe);

        // Check for magic
        if consume!(reader, u16, "DOS Magic") != DOS_SIGNITURE {
            return Err(Error::BadDOSSigniture);
        }

        // Skip ahead to PE Header Pointer
        reader.seek(SeekFrom::Start(POINTER_TO_PE_HEADER_OFFSET)).map_err(Error::Seek)?;
    
        // Get the start location of the header
        let header_pointer = consume!(reader, u32,"Pointer To PE Header");

        // Go to header start and find PE Magic bytes
        reader.seek(SeekFrom::Start(header_pointer as u64)).map_err(Error::Seek)?;
        if consume!(reader, u32, "PE Magic") != PE_SIGNITURE{
            return Err(Error::BadPESigniture);
        }

        // Get the machine type
        MachineType::try_from(consume!(reader, u16, "Machine Type"))?;

        // Get number of sections
        let num_of_sections = consume!(reader, u16, "Number of Sections");

        // Get time Date stamp (Epoch Seconds)
        consume!(reader, u32, "TimeDate Stamp");

        // Get Pointer to Symbol Table (Deprecated)
        consume!(reader, u32, "Pointer to Symbol Table");

        // Get Numeber of Symbol Table (Deprecated)
        consume!(reader, u32, "Number of Symbol Table");

        // Size Of the Optional Header
        let optional_header_size = consume!(reader, u16, "Size of Optional Header");

        // Get Characteristics
        let _characteristics = Characteristics::get(consume!(reader, u16, "Characteristics"))?;
        //println!("{:?}", characteristics);

        // Get COFF Field Magic
        OptionalHeaderMagic::try_from(consume!(reader, u16, "Optional Header Magic"))?;

        // Get MajorLinkerVersion
        consume!(reader, u8, "Major Linker Version");

        // Get MinorLinkerVersion
        consume!(reader, u8, "Minor Linker Version");

        // Get SizeOfCode
        consume!(reader, u32, "Size of the .text section");

        // Get SizeOfInitializedData
        consume!(reader, u32, "Size of the initialized data section");

        // Get SizeOfUnInitializedData
        consume!(reader, u32, "Size of the uninitialized data section (.BSS)");
        
        // Get EntryPoint Address
        let _entry = consume!(reader, u32, "Entry Point");
        //println!("PE Entry Point: {:#X} ", entry);
        // Get CodeBase
        consume!(reader, u32, "Base of Code");

        // Get DataBase
        consume!(reader, u32, "Base of Data");

        // Get image base
        let image_base = consume!(reader, u32, "Base of Image");
        //println!("Image base: {:#X}", image_base);

        // Missing the fields above, do I even need?
        // Skip to Section tabele
        reader.seek(SeekFrom::Start((header_pointer + COFF_HEADER_SIZE + optional_header_size as u32) as u64)).map_err(Error::Seek)?;

        // Store the section tables in a Vec
        let mut sections: Vec<Section> = Vec::new();

        for _ in 0..num_of_sections{
            let name = String::from_utf8(consume!(reader, u64, "Section Name")
                .to_le_bytes().to_vec()).map_err(Error::CantConvertToUtf)?;
            let virtual_size = consume!(reader, u32, "Virtual Size");
            let virtual_addr = consume!(reader, u32, "Virtual Address");
            let sizeof_rawdata = consume!(reader, u32, "Size Of Raw Data");
            let pointerto_rawdata = consume!(reader, u32, "Pointer to Raw Data");
            let pointerto_relocations = consume!(reader, u32, "Pointer to Relocations");
            let pointerto_linenumbers = consume!(reader, u32, "Pointer to Line Numbers");
            let num_of_relocations = consume!(reader, u16, "Number of Relocations");
            let num_of_linenumbers = consume!(reader, u16, "Number of Line Numbers");
            let characteristics = consume!(reader, u32, "Characteristics");

            sections.push(Section {
                name,
                virtual_size,
                virtual_addr,
                sizeof_rawdata,
                pointerto_rawdata,
                pointerto_relocations,
                pointerto_linenumbers,
                num_of_relocations,
                num_of_linenumbers,
                characteristics
            });
        }

        reader.seek(SeekFrom::Start(0)).map_err(Error::Seek)?;
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes).map_err(Error::Consume)?;

        Ok(Self {
            //entry_point: entry+image_base,
            entry_point: image_base,
            sections,
            bytes,
        })
    }
    
    fn flatten(&self) -> Result<Vec<u8>>{
        println!("{:#X?}", self.sections);
        // Creating our small binary

        let mut section_bytes: Vec<(u32, &[u8])> = vec![];

        for section in &self.sections{

            let start = section.pointerto_rawdata as usize;
            let end = start + section.virtual_size as usize;
            let vaddr = section.virtual_addr;

            let bytes = if start != 0 {
                self.bytes.get(start..end).ok_or(Error::CouldNotReadSectionData)?
            }else{
                &[0u8; 4]
            };


            section_bytes.push((vaddr, bytes));
        }

        let mut program: Vec<u8> = vec![];
        for (i, (vaddr, bytes)) in section_bytes.iter().enumerate(){
            if i == section_bytes.len() - 1 { 
                program.extend_from_slice(&bytes);
                break;
            }

            let padding_sz =  (section_bytes[i+1].0 - section_bytes[i].0) as usize;

            let program_len = program.len();
            program.extend_from_slice(&bytes);
            program.resize(padding_sz + program_len, 0);
            println!("Padding: {:#X}", padding_sz);
        }

        println!("{:X?}", section_bytes);
        println!("{:X?}", program);

        //let program = &[0u8;4];
        Ok(program)
    }
}



/// Helper for reading through file Buffer
/// Takes an expression (BufReader), A Type Big enough for the field we are passing (u32), A String describing the field
#[macro_export]
macro_rules! consume {
    ($reader:expr, $ty:ty, $field:expr) => {{
        let mut buf = [0u8; std::mem::size_of::<$ty>()];
        $reader.read_exact(&mut buf).map_err(Error::Consume)?;
        // println!("{}: LE: {:#X}, BE: {:#X}", 
        //     $field, 
        //     <$ty>::from_le_bytes(buf),
        //     <$ty>::from_be_bytes(buf),
        // );
        <$ty>::from_le_bytes(buf)
    }}
}

/// Machine Type
/// https://docs.microsoft.com/en-gb/windows/win32/debug/pe-format?redirectedfrom=MSDN#machine-types
/// 
#[repr(u16)]
enum MachineType{
    I386,
}

impl TryFrom<u16> for MachineType{

    type Error = self::Error;

    fn try_from(bytes: u16) -> Result<MachineType>{
        Ok(match bytes {
            0x14c => Self::I386,
            _ => return Err(Error::UnsupportedMachineType
                (bytes)),
        })
    }
}

#[repr(u16)]
enum OptionalHeaderMagic{
    Pe32Plus,
}

impl TryFrom<u16> for OptionalHeaderMagic{

    type Error = self::Error;

    fn try_from(bytes: u16) -> Result<OptionalHeaderMagic>{
        Ok(match bytes {
            0x10B => Self::Pe32Plus,
            _ => return Err(Error::UnsupportedOptionalHeaderMagic(bytes)),
        })
    }
}

/// https://docs.microsoft.com/en-gb/windows/win32/debug/pe-format?redirectedfrom=MSDN#characteristics
/// 
#[repr(u16)]
#[derive(Debug)]
enum Characteristics{
    RelocsStripped = 0x0001,
    ExecutableImage = 0x0002,
    LineNumsStripped = 0x0004,
    LocalSymsStripped = 0x0008,
    AggressiveWSTrim = 0x0010,
    LargeAddressAware = 0x0020,
    Reserved = 0x0040,
    BytesReversedLo = 0x0080,
    Bits32 = 0x0100,
    DebugStipped = 0x0200,
    RemovableRunFromSwap = 0x0400,
    NetRunFromSwap = 0x0800,
    System = 0x1000,
    Dll = 0x2000,
    UpSystemOnly = 0x4000,
    BytesReversedHi = 0x8000,
}

impl Characteristics{

    fn get(bytes: u16) -> Result<Vec<Characteristics>> {
        let mut characteristics = Vec::new();
        if bytes & Self::RelocsStripped as u16 == Self::RelocsStripped as u16 {
            characteristics.push(Self::RelocsStripped)
        }
        if bytes & Self::ExecutableImage as u16 == Self::ExecutableImage as u16 {
            characteristics.push(Self::ExecutableImage)
        }
        if bytes & Self::LineNumsStripped as u16 == Self::LineNumsStripped as u16 {
            characteristics.push(Self::LineNumsStripped)
        }
        if bytes & Self::LocalSymsStripped as u16 == Self::LocalSymsStripped as u16 {
            characteristics.push(Self::LocalSymsStripped)
        }
        if bytes & Self::AggressiveWSTrim as u16 == Self::AggressiveWSTrim as u16 {
            characteristics.push(Self::AggressiveWSTrim)
        }
        if bytes & Self::LargeAddressAware as u16 == Self::LargeAddressAware as u16 {
            characteristics.push(Self::LargeAddressAware)
        }
        if bytes & Self::Reserved as u16 == Self::Reserved as u16 {
            characteristics.push(Self::Reserved)
        }
        if bytes & Self::BytesReversedLo as u16 == Self::BytesReversedLo as u16 {
            characteristics.push(Self::BytesReversedLo)
        }
        if bytes & Self::Bits32 as u16 == Self::Bits32 as u16 {
            characteristics.push(Self::Bits32)
        }
        if bytes & Self::DebugStipped as u16 == Self::DebugStipped as u16 {
            characteristics.push(Self::DebugStipped)
        }
        if bytes & Self::RemovableRunFromSwap as u16 == Self::RemovableRunFromSwap as u16 {
            characteristics.push(Self::RemovableRunFromSwap)
        }
        if bytes & Self::NetRunFromSwap as u16 == Self::NetRunFromSwap as u16 {
            characteristics.push(Self::NetRunFromSwap)
        }
        if bytes & Self::System as u16 == Self::System as u16 {
            characteristics.push(Self::System)
        }
        if bytes & Self::Dll as u16 == Self::Dll as u16 {
            characteristics.push(Self::Dll)
        }        
        if bytes & Self::UpSystemOnly as u16 == Self::UpSystemOnly as u16 {
            characteristics.push(Self::UpSystemOnly)
        }        
        if bytes & Self::BytesReversedHi as u16 == Self::BytesReversedHi as u16 {
            characteristics.push(Self::BytesReversedHi)
        }
        Ok(characteristics)
    }

}

#[allow(dead_code)]
#[derive(Debug)]
struct Section{
    name: String,
    virtual_size: u32,
    virtual_addr: u32,
    sizeof_rawdata: u32,
    pointerto_rawdata: u32,
    pointerto_relocations: u32,
    pointerto_linenumbers: u32,
    num_of_relocations: u16,
    num_of_linenumbers: u16,
    characteristics: u32
}