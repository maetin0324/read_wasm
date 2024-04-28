use std::fs::File;
use std::io::Write;
use super::type_sec::FuncType;
use super::func_sec::Func;
use super::export_sec::ExportFunc;
use super::code_sec::Code;


#[derive(Debug)]
pub enum Section {
  CustomSection,
  TypeSection(Vec<FuncType>),
  ImportSection,
  FunctionSection(Vec<Func>),
  TableSection,
  MemorySection,
  GlobalSection,
  ExportSection(Vec<ExportFunc>),
  StartSection,
  ElementSection,
  CodeSection(Vec<Code>),
  DataSection,
  DataCountSection,
}

impl Section {
  pub fn match_section(section_id: u8, section_data: &Vec<u8>) -> Section{
    match section_id {
      0 => {
        println!("CustomSection");
        Section::CustomSection
      },
      1=> {
        println!("TypeSection");
        File::create("type.section").unwrap().write_all(section_data).unwrap();

        let func_types = match FuncType::parse(section_data) {
          Ok((_, func_types)) => func_types,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        
        Section::TypeSection(func_types)
      },
      2 => {
        println!("ImportSection");
        Section::ImportSection
      },
      3 => {
        println!("FunctionSection");
        File::create("function.section").unwrap().write_all(section_data).unwrap();
        let funcs = match Func::parse(section_data) {
          Ok((_, funcs)) => funcs,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        Section::FunctionSection(funcs)
      },
      4 => {
        println!("TableSection");
        Section::TableSection
      },
      5 => {
        println!("MemorySection");
        Section::MemorySection
      },
      6 => {
        println!("GlobalSection");
        Section::GlobalSection
      },
      7 => {
        println!("ExportSection");
        File::create("export.section").unwrap().write_all(section_data).unwrap();
        let export_funcs = match ExportFunc::parse(section_data) {
          Ok((_, export_funcs)) => export_funcs,
          Err(e) => panic!("Error: {:#x?}", e),
        };

        Section::ExportSection(export_funcs)
      },
      8 => {
        println!("StartSection");
        Section::StartSection
      },
      9 => {
        println!("ElementSection");
        Section::ElementSection
      },
      10 => {
        println!("CodeSection");
        File::create("code.section").unwrap().write_all(section_data).unwrap();
        let codes = match Code::parse(section_data){
          Ok((_, codes)) => codes,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        // codes.iter().for_each(|code| {
        //   code.instrs.iter().for_each(|instr| {
        //     println!("{:?}", instr);
        //   });
        // });
        Section::CodeSection(codes)
      },
      11 => {
        println!("DataSection");
        Section::DataSection
      },
      12 => {
        println!("DataCountSection");
        Section::DataCountSection
      },
      _ => panic!("Unknown section id: {}", section_id),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_match_type_section() {
    let section_id = 1;
    let section_data = vec![0x01, 0x60, 0x01, 0x7f, 0x00];
    let section = Section::match_section(section_id, &section_data);
    match section {
      Section::TypeSection(func_types) => {
        assert_eq!(func_types.len(), 1);
        assert_eq!(func_types[0].param_types.len(), 1);
        assert_eq!(func_types[0].return_types.len(), 0);
      },
      _ => panic!("Invalid section: {:?}", section),
    }
  }

  #[test]
  fn test_match_code_section() {
    let section_id = 10;
    let section_data = vec![0x01, 0x09, 0x01, 0x01, 0x7e, 0x42, 0x01, 0x42, 0x02, 0x7c, 0x0b];
    let section = Section::match_section(section_id, &section_data);
    match section {
      Section::CodeSection(codes) => {
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].locals.len(), 1);
        assert_eq!(codes[0].instrs.len(), 3);
      },
      _ => panic!("Invalid section: {:?}", section),
    }
  }
}