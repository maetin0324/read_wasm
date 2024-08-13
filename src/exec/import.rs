use anyhow::{anyhow, Result, Ok};
use std::{collections::HashMap, io::Write, sync::Arc};
use super::{store::{MemoryInst, Store}, value::Value, wasi::WasiSnapshotPreview1};

pub type ImportFunc = Box<dyn FnMut(&mut WasiSnapshotPreview1, &mut Store, Vec<Value>) -> Result<Option<Value>>>;
pub type ImportTable = HashMap<String, HashMap<String, ImportFunc>>;

// static IMPORT_FUNCS: LazyLock<Mutex<Arc<Box<ImportTable>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn init_import() -> ImportTable {
  let mut import: ImportTable = HashMap::new();
  let mut add_hash: HashMap<String, ImportFunc> = HashMap::new();
  add_hash.insert("add".to_owned(), Box::new(|_, _, values| ->Result<Option<Value>> {
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

pub fn fd_write(wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();

  let fd = args[0];
  let mut iovs = args[1] as usize;
  let iovs_len = args[2];
  let rp = args[3] as usize;

  let file = wasi
    .file_table
    .get_mut(fd as usize)
    .ok_or(anyhow::anyhow!("not found fd"))?;

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

pub fn random_get(_wasi: WasiSnapshotPreview1,store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let buf = args[0] as usize;
  let buf_len = args[1] as usize;
  for i in 0..buf_len {
      let random = rand::random();
      store.memories[0].memory[buf + i] = random;
  }
  Ok(Some(Value::I32(0)))
}

fn fd_prestat_get(wasi: WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let fd = args[0];
  let buf = args[1] as u32;

  let Some(Some(path)) = wasi.file_path.get(fd as usize) else {
      return Ok(Some(ERRNO_BADF.into()));
  };
  store.memories[0].store(buf as u32, 0, 1, &[0])?;
  store
      .memories[0]
      .store(buf, 4, 4, &(path.len() as i32).to_le_bytes())?;
  Ok(Some(Value::I32(0)))
}

const RIGHTS_FD_READ: i64 = 2;
const RIGHTS_FD_READDIR: i64 = 0x4000;
const RIGHTS_FD_DATASYNC: i64 = 0x1;
const RIGHTS_FD_WRITE: i64 = 0x40;
const RIGHTS_FD_ALLOCATE: i64 = 0x100;
const RIGHTS_FD_FILESTAT_SET_SIZE: i64 = 0x400000;
const FDFLAGS_APPEND: i32 = 0x1;
const OFLAGS_CREAT: i32 = 0x1;
const OFLAGS_EXCL: i32 = 0x4;
const OFLAGS_TRUNC: i32 = 0x8;
const FILETYPE_UNKNOWN: u8 = 0;
const FILETYPE_DIRECTORY: u8 = 3;
const FILETYPE_REGULAR_FILE: u8 = 4;
const FILETYPE_SYMBOLIC_LINK: u8 = 7;

const ERRNO_BADF: i32 = 8;
const ERRNO_INVAL: i32 = 28;


