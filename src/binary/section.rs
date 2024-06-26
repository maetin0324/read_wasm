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
  pub fn match_section(section_id: u8, section_data: &[u8]) -> Section{
    match section_id {
      0 => {
        Section::CustomSection
      },
      1 => {
        let func_types = match FuncType::parse(section_data) {
          Ok((_, func_types)) => func_types,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        
        Section::TypeSection(func_types)
      },
      2 => {
        Section::ImportSection
      },
      3 => {
        let funcs = match Func::parse(section_data) {
          Ok((_, funcs)) => funcs,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        Section::FunctionSection(funcs)
      },
      4 => {
        Section::TableSection
      },
      5 => {
        Section::MemorySection
      },
      6 => {
        Section::GlobalSection
      },
      7 => {
        let export_funcs = match ExportFunc::parse(section_data) {
          Ok((_, export_funcs)) => export_funcs,
          Err(e) => panic!("Error: {:#x?}", e),
        };

        Section::ExportSection(export_funcs)
      },
      8 => {
        Section::StartSection
      },
      9 => {
        Section::ElementSection
      },
      10 => {
        let codes = match Code::parse(section_data){
          Ok((_, codes)) => codes,
          Err(e) => panic!("Error: {:#x?}", e),
        };
        Section::CodeSection(codes)
      },
      11 => {
        Section::DataSection
      },
      12 => {
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