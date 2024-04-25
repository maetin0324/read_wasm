use std::fs::File;
use std::io::Write;
use super::code::Code;

pub enum Section {
  CustomSection,
  TypeSection,
  ImportSection,
  FunctionSection,
  TableSection,
  MemorySection,
  GlobalSection,
  ExportSection,
  StartSection,
  ElementSection,
  CodeSection,
  DataSection,
  DataCountSection,
}

impl Section {
  pub fn match_section(section_id: u8, section_data: &Vec<u8>) {
    match Section::from_id(section_id) {
      Section::CustomSection => {
        println!("CustomSection");
      },
      Section::TypeSection => {
          println!("TypeSection");
          File::create("type.section").unwrap().write_all(section_data).unwrap();
      },
      Section::ImportSection => {
          println!("ImportSection");
      },
      Section::FunctionSection => {
          println!("FunctionSection");
          File::create("function.section").unwrap().write_all(section_data).unwrap();
      },
      Section::TableSection => {
          println!("TableSection");
      },
      Section::MemorySection => {
          println!("MemorySection");
      },
      Section::GlobalSection => {
          println!("GlobalSection");
      },
      Section::ExportSection => {
          println!("ExportSection");
          File::create("export.section").unwrap().write_all(section_data).unwrap();
      },
      Section::StartSection => {
          println!("StartSection");
      },
      Section::ElementSection => {
          println!("ElementSection");
      },
      Section::CodeSection => {
          println!("CodeSection");
          File::create("code.section").unwrap().write_all(section_data).unwrap();
          let (_, codes) = Code::parse(section_data).unwrap();
          codes.iter().enumerate().for_each(|(i, code)| {
            println!("Function #{}: size: {}", i, code.size);
          });
      },
      Section::DataSection => {
          println!("DataSection");
      },
      Section::DataCountSection => {
          println!("DataCountSection");
      },
    }
  }
  fn from_id(section_id: u8) -> Section {
    match section_id {
        0 => Section::CustomSection,
        1 => Section::TypeSection,
        2 => Section::ImportSection,
        3 => Section::FunctionSection,
        4 => Section::TableSection,
        5 => Section::MemorySection,
        6 => Section::GlobalSection,
        7 => Section::ExportSection,
        8 => Section::StartSection,
        9 => Section::ElementSection,
        10 => Section::CodeSection,
        11 => Section::DataSection,
        12 => Section::DataCountSection,
        _ => panic!("Unknown section id: {}", section_id),
    }
  }
}