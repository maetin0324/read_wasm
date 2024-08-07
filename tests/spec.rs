#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use std::fs::File;
use std::io::Read;
use std::{path, vec};

  use read_wasm::binary;
  use read_wasm::binary::wasm::Wasm;
  use read_wasm::exec::exec_machine::ExecMachine;
  use read_wasm::exec::func_instance::FuncInstance;
use read_wasm::exec::value::Value;

  fn create_wasm_from_testsuite(path: &str) -> Wasm {
    let mut test_suite = String::new();
    File::open(path).unwrap()
      .read_to_string(&mut test_suite).unwrap();
    let binary = wat::parse_str(&test_suite).unwrap();
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

  #[test]
  fn test_parse_importsec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/import.wat");
    assert!(wasm.import_section.is_some());
    assert!(wasm.function_section.is_none());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());

    let imports = wasm.import_section.unwrap();
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].module, "env");
    assert_eq!(imports[0].field, "one");
    assert_eq!(imports[0].desc, binary::import_sec::ImportDesc::Func(0));
  }

  #[test]
  fn test_parse_memorysec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/memorysec.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.import_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.memory_section.is_some());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());

    let memories = wasm.memory_section.unwrap();
    assert_eq!(memories[0].min, 0);
    assert_eq!(memories[0].max, Some(10));
  }

  #[tokio::test]
  async fn test_instantiate_with_import() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/func.wat");
    assert!(wasm.import_section.is_some());
    assert!(wasm.function_section.is_some());
    assert!(wasm.export_section.is_some());
    assert!(wasm.code_section.is_some());

    let func_instances = FuncInstance::new(&wasm);
    assert_eq!(func_instances.len(), 3);
    assert_eq!(func_instances[0].name().unwrap(), "one");
    assert_eq!(func_instances[1].name().unwrap(), "none");
    assert_eq!(func_instances[2].name().unwrap(), "_start");

    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    em.exec().await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
  }

  #[tokio::test]
  async fn test_import_func() {
    let wasm = create_wasm_from_testsuite("tests/testsuite/import_func.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    em.exec().await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
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