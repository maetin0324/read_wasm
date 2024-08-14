#![allow(dead_code)]

use anyhow::{anyhow, Result, Ok};
use std::{collections::HashMap, env, fs::OpenOptions, io::{Read, Seek, SeekFrom, Write}, mem::ManuallyDrop, path::Path};
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
  wasi_hash.insert("random_get".to_owned(), Box::new(random_get));
  wasi_hash.insert("fd_prestat_get".to_owned(), Box::new(fd_prestat_get));
  wasi_hash.insert("fd_prestat_dir_name".to_owned(), Box::new(fd_prestat_dir_name));
  wasi_hash.insert("fd_close".to_owned(), Box::new(fd_close));
  wasi_hash.insert("fd_read".to_owned(), Box::new(fd_read));
  wasi_hash.insert("environ_sizes_get".to_owned(), Box::new(environ_sizes_get));
  wasi_hash.insert("environ_get".to_owned(), Box::new(environ_get));
  wasi_hash.insert("path_open".to_owned(), Box::new(path_open));
  wasi_hash.insert("fd_seek".to_owned(), Box::new(fd_seek));


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
    .ok_or(anyhow::anyhow!("not found fd"))?
    .as_mut()
    .ok_or(anyhow!("not found fd"))?;

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

fn random_get(_wasi: &mut WasiSnapshotPreview1,store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let buf = args[0] as usize;
  let buf_len = args[1] as usize;
  for i in 0..buf_len {
      let random = rand::random();
      store.memories[0].memory[buf + i] = random;
  }
  Ok(Some(Value::I32(0)))
}

fn fd_prestat_get(wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let fd = args[0];
  let buf = args[1] as u32;

  let Some(Some(path)) = wasi.file_path.get(fd as usize) else {
      return Ok(Some(ERRNO_BADF.into()));
  };
  store.memories[0].store(buf, 0, 1, &[0])?;
  store
      .memories[0]
      .store(buf, 4, 4, &(path.len() as i32).to_le_bytes())?;
  Ok(Some(Value::I32(0)))
}

fn fd_prestat_dir_name(
  wasi: &mut WasiSnapshotPreview1,
  store: &mut Store,
  args: Vec<Value>,
) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let fd = args[0] as usize;
  let buf = args[1] as usize;

  let Some(Some(path)) = wasi.file_path.get(fd) else {
      return Ok(Some(ERRNO_BADF.into()));
  };
  for i in 0..path.len() {
      store.memories[0].memory[buf + i] = path.as_bytes()[i];
  }
  Ok(Some(Value::I32(0)))
}

fn fd_close(wasi: &mut WasiSnapshotPreview1, _store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let fd = args[0] as usize;
  if fd >= 3 {
      wasi.file_table[fd] = None;
      wasi.file_path[fd] = None;
  }
  Ok(Some(Value::I32(0)))
}

fn fd_read(wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args = args.into_iter().map(Into::into).collect::<Vec<i32>>();
  let fd = args[0];
  let mut iovs = args[1] as u32;
  let iovs_len = args[2];
  let rp = args[3];

  let file = wasi
      .file_table
      .get_mut(fd as usize)
      .ok_or(anyhow!("Not found fd"))?
      .as_mut()
      .ok_or(anyhow!("Not found fd"))?;

  let mut nread = 0;
  let memory = &mut store.memories[0];

  for _ in 0..iovs_len {
      let start = memory_read_4byte(memory, iovs)? as usize;
      iovs += 4;
      let len = memory_read_4byte(memory, iovs)? as usize;
      iovs += 4;
      let end = start + len;
      nread += file
          .read(&mut memory.memory[start..end])?;
  }
  memory.store(rp as u32, 0, 4, &nread.to_le_bytes())?;

  Ok(Some(Value::I32(0)))
}

fn environ_sizes_get(_wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let environc_offset = args[0] as u32;
  let environ_buf_size_offset = args[1] as u32;
  let mut environc: i32 = 0;
  let mut environ_buf_size: i32 = 0;
  for env in env::vars() {
      environc += 1;
      environ_buf_size += (env.0.len() + 1 + env.1.len() + 1) as i32;
  }
  let memory = &mut store.memories[0];
  memory.store(environc_offset, 0, 4, &environc.to_le_bytes())?;
  memory.store(
      environ_buf_size_offset,
      0,
      4,
      &environ_buf_size.to_le_bytes(),
  )?;
  Ok(Some(Value::I32(0)))
}

fn environ_get(_wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let args: Vec<i32> = args.into_iter().map(Into::into).collect();
  let mut environ_offset = args[0] as u32;
  let mut environ_buf_offset = args[1] as i32;
  for (key, value) in env::vars() {
      store
          .memories[0]
          .store(environ_offset, 0, 4, &environ_buf_offset.to_le_bytes())?;
      environ_offset += 4;
      let text = format!("{key}={value}\0");
      store.memories[0].store(
          environ_buf_offset as u32,
          0,
          text.len() as u32,
          text.as_bytes(),
      )?;
      environ_buf_offset += text.len() as i32;
  }
  Ok(Some(Value::I32(0)))
}

fn path_open(wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let fd: i32 = args[0].clone().into();
  // let dirflags = args[1].;
  let path_offset = i32::from(args[2].clone()) as u32;
  let path_len: i32 = args[3].clone().into();
  let oflags: i32 = args[4].clone().into();
  let rights_base: i64 = args[5].clone().into();
  // let rights_inheriting = args[6].as_i64()?;
  let fdflags: i32 = args[7].clone().into();
  let opened_fd_offset = i32::from(args[8].clone()) as u32;

  let Some(Some(path)) = wasi.file_path.get(fd as usize) else {
      return Ok(Some(ERRNO_INVAL.into()));
  };

  let file_path = store
      .memories[0]
      .load(path_offset, 0, path_len as u32)?
      .into_iter()
      .map(|b| *b as char)
      .collect::<String>();
  let file_path = file_path.trim_matches('\0');
  let resolved_path = Path::new(path).join(file_path);
  let open_options = OpenOptions::new()
    .create((oflags & OFLAGS_CREAT) != 0)
    .truncate((oflags & OFLAGS_TRUNC) != 0)
    .create_new((oflags & OFLAGS_EXCL) != 0)
    .read((rights_base & (RIGHTS_FD_READ | RIGHTS_FD_READDIR)) != 0)
    .write(
      (rights_base
          & (RIGHTS_FD_DATASYNC
              | RIGHTS_FD_WRITE
              | RIGHTS_FD_ALLOCATE
              | RIGHTS_FD_FILESTAT_SET_SIZE))
          != 0,
    )
    .append((fdflags & FDFLAGS_APPEND) != 0)
    .open(&resolved_path)?;
  wasi.file_table.push(Some(Box::new(ManuallyDrop::new(open_options))));
  let opened_fd = wasi.file_table.len() as i32 - 1;
  wasi.file_path
    .push(Some(resolved_path.to_str().unwrap().to_string()));
  store
    .memories[0]
    .store(opened_fd_offset, 0, 4, &opened_fd.to_le_bytes())?;

  Ok(Some(Value::I32(0)))
}

fn fd_seek(wasi: &mut WasiSnapshotPreview1, store: &mut Store, args: Vec<Value>) -> Result<Option<Value>> {
  let fd: i32 = args[0].clone().into();
  let offset = args[1].clone().into();
  let whence = args[2].clone().into();
  let new_offset_offset: i32 = args[3].clone().into();

  let Some(Some(file)) = wasi.file_table.get_mut(fd as usize) else {
    return Ok(Some(ERRNO_BADF.into()));
  };

  let new_offset = match whence {
    0 => file.seek(SeekFrom::Start(offset as u64)),
    1 => file.seek(SeekFrom::Current(offset)),
    2 => file.seek(SeekFrom::End(offset)),
    _ => return Ok(Some(ERRNO_INVAL.into())),
  }?;

  store
    .memories[0]
    .store(new_offset_offset as u32, 0, 8, &new_offset.to_le_bytes())?;

  Ok(Some(Value::I32(0)))
}

fn memory_read_4byte(memory: &MemoryInst, addr: u32) -> Result<i32> {
  let bytes = memory.load(addr, 0, 4)?;
  Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
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


