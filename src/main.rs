use std::fs::File;
use std::io::{BufReader, Read};
use clap::Parser;
use read_wasm::binary::section::Section;

#[derive(Parser, Debug)]
struct Cli {
    filename: String,
}

const MAGIC: u32 = 0x0061736d;
const VERSION: u32 = 0x01000000;

fn main() {
    let args = Cli::parse();
    let filename = args.filename;
    
    let file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);

    let (magic, version) = read_magic_and_version(&mut reader);
    if magic != MAGIC {
        print!("magic is {:x}, expected {:x}", magic, MAGIC);
        panic!("Magic number is not correct");
    }
    if version != VERSION {
        print!("version is {:x}, expected {:x}", version, VERSION);
        panic!("Version number is not correct");
    }

    loop {
        let (section_id, section_size) = read_section_id_and_size(&mut reader);
        if section_id == 0 && section_size == 0 {
            break;
        }
        println!("section_id: {}, section_size: {}", section_id, section_size);
        let mut section_data = vec![0u8; section_size as usize];
        reader.read_exact(&mut section_data).unwrap();

        Section::match_section(section_id, &section_data)
    }
}

// セクションパース部分nomで書き直せそうだが、気合で書いて愛着があるので一旦このまま

fn read_u32_from_leb128<T: Read>(reader: &mut BufReader<T>) -> u32 {
    let mut acc: u32 = 0;
    let mut count: u8 = 0;
    for byte in reader.bytes() {
        if let Ok(b) = byte {
            let val: u32 = (b & 0b01111111) as u32;
            let shifted_val = val << (7 * count);
            acc += shifted_val as u32;
            count += 1;
            if b < 0b10000000 { break; }
        } else {
            break;
        }
    }
    acc
}

fn read_magic_and_version<T: Read>(reader: &mut BufReader<T>) -> (u32, u32) {
    let mut magic_buf = vec![0u8; 4];
    let mut version_buf = vec![0u8; 4];
    reader.read_exact(&mut magic_buf).unwrap();
    reader.read_exact(&mut version_buf).unwrap();
    let magic = vecu8_to_u32(magic_buf);
    let version = vecu8_to_u32(version_buf);

    (magic, version)
}

fn vecu8_to_u32(buf: Vec<u8>) -> u32 {
    let mut acc: u32 = 0;
    for (i, byte) in buf.iter().enumerate() {
        acc += (*byte as u32) << ((3 - i) * 8);
    }
    acc
}

fn read_section_id_and_size<T: Read>(reader: &mut BufReader<T>) -> (u8, u32) {
    // if read EOF, return (0, 0)
    let mut bytes = reader.bytes();
    match bytes.next() {
        Some(Ok(b)) => {
            let section_id = b;
            let section_size = read_u32_from_leb128(reader);
            (section_id, section_size)
        },
        _ => (0, 0),
    }
}
