use anyhow::{Result, Ok};
use std::{collections::HashMap, io::Write, sync::Arc};
use super::{store::Store, value::Value, wasi::WasiSnapshotPreview1};

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

  let mut wasi_hash: HashMap<String, ImportFunc> = HashMap::new();
  wasi_hash.insert("fd_write".to_owned(), Box::new(fd_write));

  import.insert("wasi_snapshot_preview1".to_owned(), wasi_hash);

  import
}

pub fn fd_write(store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();

  let fd = args[0];
  let mut iovs = args[1] as usize;
  let iovs_len = args[2];
  let rp = args[3] as usize;

  let mut wasi = WasiSnapshotPreview1::new();

  let file = wasi
    .file_table
    .get_mut(fd as usize)
    .ok_or(anyhow::anyhow!("not found fd"))?;

  let file = Arc::clone(file);
  let mut file = file.lock().expect("cannot lock file");

  let memory = store
    .memories
    .get_mut(0)
    .ok_or(anyhow::anyhow!("not found memory"))?;

  let mut nwritten = 0;

  for _ in 0..iovs_len {
    let start = memory_read(&memory.memory, iovs)? as usize;
    iovs += 4;

    let len: i32 = memory_read(&memory.memory, iovs)?;
    iovs += 4;

    let end = start + len as usize;
    nwritten += file.write(&memory.memory[start..end])?;
  }

  memory_write(&mut memory.memory, rp, &nwritten.to_le_bytes())?;

  Ok(Some(0.into()))
}

fn memory_read(buf: &[u8], start: usize) -> Result<i32> {
  let end = start + 4;
  Ok(<i32>::from_le_bytes(buf[start..end].try_into()?))
}

fn memory_write(buf: &mut [u8], start: usize, data: &[u8]) -> Result<()> {
  let end = start + data.len();
  buf[start..end].copy_from_slice(data);
  Ok(())
}



