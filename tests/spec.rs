#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::Read;
  use std::{path, vec};

  use read_wasm::binary;
  use read_wasm::binary::table_sec::RefType;
use read_wasm::binary::wasm::Wasm;
  use read_wasm::exec::exec_machine::ExecMachine;
  use read_wasm::exec::func_instance::FuncInstance;
  use read_wasm::exec::store::Store;
  use read_wasm::exec::value::Value;
  use read_wasm::exec::wasi::WasiSnapshotPreview1;

  fn create_wasm_from_testsuite(path: &str) -> Wasm {
    let mut test_suite = String::new();
    File::open(path).unwrap()
      .read_to_string(&mut test_suite).unwrap();
    let binary = wat::parse_str(&test_suite).unwrap();
    Wasm::new(&binary[..])
  }

  #[test]
  fn test_parse_module_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/module.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());
  }

  #[test]
  fn test_parse_typesec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/typesec.wat");
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
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/import.wat");
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
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/memorysec.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.import_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.memory_section.is_some());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());

    let memories = wasm.memory_section.unwrap();
    assert_eq!(memories[0].min, 1);
    assert_eq!(memories[0].max, Some(10));
  }

  #[test]
  fn test_parse_datasec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/datasec.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.import_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.memory_section.is_some());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());
    assert!(wasm.data_section.is_some());

    let data = wasm.data_section.unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].memory_index, 0);
    assert_eq!(data[0].offset, 1);
    assert_eq!(&data[0].init, &[0x61, 0x62, 0x63, 0x64])
  }

  #[test]
  fn test_parse_globalsec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/globalsec.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.import_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.memory_section.is_none());
    assert!(wasm.export_section.is_none());
    assert!(wasm.code_section.is_none());
    assert!(wasm.global_section.is_some());

    let globals = wasm.global_section.unwrap();
    assert_eq!(globals.len(), 1);
    assert_eq!(globals[0].valtype, binary::value_type::ValueType::I32);
    assert!(!globals[0].mutability);
  }

  #[test]
  fn test_parse_table_sec_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/tablesec.wat");
    assert!(wasm.type_section.is_none());
    assert!(wasm.import_section.is_none());
    assert!(wasm.function_section.is_none());
    assert!(wasm.memory_section.is_none());
    assert!(wasm.export_section.is_some());
    assert!(wasm.code_section.is_none());
    assert!(wasm.table_section.is_some());

    let tables = wasm.table_section.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].min, 1);
    assert_eq!(tables[0].max, Some(1));
    assert_eq!(tables[0].reftype, RefType::FuncRef);
  }

  #[test]
  fn test_init_memory_store() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/memorysec.wat");
    let store = Store::new(vec![], &wasm);
    assert_eq!(store.memories.len(), 1);
    assert_eq!(store.memories[0].memory.len(), 65536)
  }

  #[test]
  fn test_init_data() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/data.wat");
    let store = Store::new(vec![], &wasm);
    assert_eq!(store.memories.len(), 1);
    assert_eq!(store.memories[0].memory.len(), 65536);
    assert_eq!(&store.memories[0].memory[0..5], b"hello");
    assert_eq!(&store.memories[0].memory[5..10], b"world");
  }

  #[tokio::test]
  async fn test_instantiate_with_import() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/func.wat");
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
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
  }

  #[tokio::test]
  async fn test_import_func() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/import_func.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
  }

  #[tokio::test]
  async fn test_exec_add_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/add.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(3));
  }

  #[tokio::test]
  async fn test_exec_block_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/block.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![Value::I64(100)]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I64(5050));
  }

  #[tokio::test]
  async fn test_exec_block_table_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/block_table.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I32(213));
  }

  #[tokio::test]
  async fn test_i32_store_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/i32store.wat");
    let mut em = ExecMachine::init(wasm, "i32_store",vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    let memory = &em.store.memories[0].memory;
    assert_eq!(memory[0], 42);
  }

  #[tokio::test]
  async fn test_i64_store_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/i64store.wat");
    let mut em = ExecMachine::init(wasm, "i64_store",vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    let memory = &em.store.memories[0].memory;
    assert_eq!(memory[0], 42);
  }

  #[tokio::test]
  async fn test_global_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/global.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    assert_eq!(em.value_stack.last().unwrap(), &Value::I32(50));
  }


  #[tokio::test]
  async fn test_hello_world_wasm() {
    let wasm = create_wasm_from_testsuite("tests/mytestsuite/hello_world.wat");
    let mut em = ExecMachine::init(wasm, "_start", vec![]);
    let mut wasi = WasiSnapshotPreview1::new();
    em.exec(&mut wasi).await.unwrap();
    let memory = &em.store.memories[0].memory;
    let hello = &memory[0..6];
    let world = &memory[6..13];
    assert_eq!(hello, b"Hello,");
    assert_eq!(world, b" World!");
    assert_eq!(em.value_stack.last().unwrap(), &Value::I32(0));
  }
}