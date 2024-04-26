use super::type_sec::FuncType;
use super::export_sec::ExportFunc;
use super::code_sec::Code;

// parse結果を格納する構造体
#[derive(Debug, PartialEq)]
pub struct Wasm {
  pub type_section: Option<Vec<FuncType>>,
  pub export_section: Option<Vec<ExportFunc>>,
  pub code_section: Option<Vec<Code>>,
}

impl Wasm {
  pub fn get_code_by_name(&self, name: &str) -> Option<&Code> {
    self.export_section.as_ref().and_then(|export_section| {
      export_section.iter().find_map(|export_func| {
        if export_func.name == name {
          self.code_section.as_ref().and_then(|code_section| {
            code_section.get(export_func.func_idx as usize)
          })
        } else {
          None
        }
      })
    })
  }
}