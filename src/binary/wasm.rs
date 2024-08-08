use std::io::Read;
use nom::bytes::complete::{tag, take};
use nom::IResult;
use nom_leb128::leb128_u32;

use super::data_sec::Data;
use super::memory_sec::MemorySec;
use super::section::Section;
use super::type_sec::FuncType;
use super::import_sec::Import;
use super::func_sec::Func;
use super::export_sec::ExportFunc;
use super::code_sec::Code;

// parse結果を格納する構造体
#[derive(Debug, PartialEq)]
pub struct Wasm {
  pub type_section: Option<Vec<FuncType>>,
  pub import_section: Option<Vec<Import>>,
  pub function_section: Option<Vec<Func>>,
  pub memory_section: Option<Vec<MemorySec>>,
  pub export_section: Option<Vec<ExportFunc>>,
  pub code_section: Option<Vec<Code>>,
  pub data_section: Option<Vec<Data>>
}

const MAGIC: &[u8; 4] = &[0x00, 0x61, 0x73, 0x6d];
const VERSION: &[u8; 4] = &[0x01, 0x00, 0x00, 0x00];

impl Wasm {
  pub fn new<T: Read>(mut reader: T) -> Wasm {

    let mut all_data: Vec<u8> = Vec::new();
    reader.read_to_end(&mut all_data).unwrap();

    let mut data = check_magic_and_version(&all_data);

    let mut wasm = Wasm{
      type_section: None,
      import_section: None,
      function_section: None,
      memory_section: None,
      export_section: None,
      code_section: None,
      data_section: None,
    };

    loop {
      let section_id: u8;
      let section_size: u32;
      let section_data: &[u8];

      (data, (section_id, section_size, section_data)) = parse_section_id_and_content(data).unwrap();
      if section_id == 0 && section_size == 0 {
          break;
      }

      let section = Section::match_section(section_id, section_data);
      match section {
        Section::TypeSection(func_types) => {
            wasm.type_section = Some(func_types);
        },
        Section::ImportSection(imports) => {
            wasm.import_section = Some(imports);
        },
        Section::FunctionSection(funcs) => {
            wasm.function_section = Some(funcs);
        },
        Section::MemorySection(memories) => {
          wasm.memory_section = Some(memories);
        },
        Section::ExportSection(export_funcs) => {
            wasm.export_section = Some(export_funcs);
        },
        Section::CodeSection(codes) => {
            wasm.code_section = Some(codes);
        },
        Section::DataSection(data) => {
          wasm.data_section = Some(data);
        },
        _ => {},
      }
    }
    wasm
  }
}

fn check_magic_and_version(data: &[u8]) -> &[u8] {
  let data = match tag::<_, _, ()>(MAGIC)(data) {
    Ok((data, _)) => data,
    Err(e) => panic!("Unexpected magic number: {:#x?}", e),
  };
  
  (match tag::<_, _, ()>(VERSION)(data) {
    Ok((data, _)) => data,
    Err(e) => panic!("Unexpected version number: {:#x?}", e),
  }) as _
}


fn parse_section_id_and_content(data: &[u8]) -> IResult<&[u8], (u8, u32, &[u8])> {
  if data.is_empty() {
    return Ok((data, (0, 0, &[])));
  }
  let (data, section_id) = take(1u8)(data)?;
  let (data, section_size) = leb128_u32(data)?;
  let (data, section_data) = take(section_size)(data)?;
  Ok((data, (section_id[0], section_size, section_data)))
}

// セクションパース部分nomで書き直せそうだが、気合で書いて愛着があるので一旦このまま

// fn read_u32_from_leb128<T: Read>(reader: T) -> u32 {
//   let mut acc: u32 = 0;
//   let mut count: u8 = 0;
//   for byte in reader.bytes() {
//       if let Ok(b) = byte {
//           let val: u32 = (b & 0b01111111) as u32;
//           let shifted_val = val << (7 * count);
//           acc += shifted_val;
//           count += 1;
//           if b < 0b10000000 { break; }
//       } else {
//           break;
//       }
//   }
//   acc
// }

// fn read_magic_and_version<T: Read>(mut reader: T) -> (u32, u32) {
//   let mut magic_buf = vec![0u8; 4];
//   let mut version_buf = vec![0u8; 4];
//   reader.read_exact(&mut magic_buf).unwrap();
//   reader.read_exact(&mut version_buf).unwrap();
//   let magic = vecu8_to_u32(magic_buf);
//   let version = vecu8_to_u32(version_buf);

//   (magic, version)
// }

// fn vecu8_to_u32(buf: Vec<u8>) -> u32 {
//   let mut acc: u32 = 0;
//   for (i, byte) in buf.iter().enumerate() {
//       acc += (*byte as u32) << ((3 - i) * 8);
//   }
//   acc
// }

// fn read_section_id_and_size<T: Read>(reader: &mut BufReader<T>) -> (u8, u32) {
//   // if read EOF, return (0, 0)
//   let mut bytes = reader.bytes();
//   match bytes.next() {
//       Some(Ok(b)) => {
//           let section_id = b;
//           let section_size = read_u32_from_leb128(reader);
//           (section_id, section_size)
//       },
//       _ => (0, 0),
//   }
// }