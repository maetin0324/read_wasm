#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use std::fs::File;
use std::io::Read;
use std::path;

  use read_wasm::binary;
  use read_wasm::binary::wasm::Wasm;
  use read_wasm::exec::exec_machine::ExecMachine;
  use read_wasm::exec::value::Value;
  use wat::parse_str;

  fn create_wasm_from_testsuite(path: &str) -> Wasm {
    let mut test_suite = String::new();
    File::open(path).unwrap()
      .read_to_string(&mut test_suite).unwrap();
    let binary = parse_str(&test_suite).unwrap();
    Wasm::new(&binary[..])
  }

  #[test]
  fn test_parse_module_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/module.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());
  }

  #[test]
  fn test_parse_typesec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/typesec.wat");
    assert!(wasm.type_section.is_some());
    assert!(wasm.function_section.is_none());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());

    let types = wasm.type_section.unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].param_types, vec![
      binary::value_type::ValueType::I32,
      binary::value_type::ValueType::I32
    ]);
    assert_eq!(types[0].return_types, vec![binary::value_type::ValueType::I32]);
  }

  #[tokio::test]
  async fn test_exec_add_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/add.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    em.exec().await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
  }

  #[tokio::test]
  async fn test_exec_block_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/block.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![Value::I64(100)]);
    em.exec().await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(5050));
  }
}