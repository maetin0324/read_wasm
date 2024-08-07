use anyhow::{Result, Ok};
use std::collections::HashMap;
use super::{store::Store, value::Value};

pub type ImportFunc = Box<dyn FnMut(&mut Store, Vec<Value>) -> Result<Option<Value>>>;
pub type ImportTable = HashMap<String, HashMap<String, ImportFunc>>;

// static IMPORT_FUNCS: LazyLock<Mutex<Arc<Box<ImportTable>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn init_import() -> ImportTable {
  let mut import: ImportTable = HashMap::new();
  let mut add_hash: HashMap<String, ImportFunc> = HashMap::new();
  add_hash.insert("add".to_owned(), Box::new(|_, values| ->Result<Option<Value>> {
    match (values[0].clone(), values[1].clone()) {
      (Value::I64(a), Value::I64(b)) => Ok(Some(Value::I64(a + b))),
      _ => panic!("Invalid arg types in import func")
    }
  }));
  import.insert("env".to_owned(), add_hash);

  import

}




