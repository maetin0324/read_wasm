use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Result};
use crate::binary::{instructions::Instructions, wasm::Wasm};
use super::{func_instance::FuncInstance, value::Value};

pub const PAGE_SIZE: usize = 65536; // 64Ki

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct Store {
  pub funcs: Vec<FuncInstance>,
  pub memories: Vec<MemoryInst>,
  pub globals: Vec<GlobalValue>,
}

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct MemoryInst {
  pub memory: Vec<u8>,
  pub max: Option<u32>,
}

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct GlobalValue {
  pub value: Value,
  pub mutability: bool,
}

impl Store {
  pub fn new(funcs: Vec<FuncInstance>, wasm: &Wasm) -> Store {
    let mut memories = Vec::new();
    if let Some(ref memory_sec) = wasm.memory_section {
      for memory in memory_sec {
        let min = memory.min * PAGE_SIZE as u32;
        let memory_inst = MemoryInst {
          memory: vec![0; min as usize],
          max: memory.max,
        };
        memories.push(memory_inst);
      }
    }

    if let Some(ref data) = wasm.data_section {
      for data in data {
        let memory = memories
          .get_mut(data.memory_index as usize)
          .unwrap();
        let offset = data.offset as usize;
        let init = &data.init;

        if offset + init.len() > memory.memory.len() {
            panic!("data is too large to fit in memory");
        }
        memory.memory[offset..offset + init.len()].copy_from_slice(init);
      }     
    }

    let mut globals = Vec::new();
    if let Some(ref global_sec) = wasm.global_section {
      for global in global_sec {
        let value = match global.init[0] {
          Instructions::I32Const(v) => Value::I32(v),
          Instructions::I64Const(v) => Value::I64(v),
          _ => panic!("Invalid global init value"),
        };
        globals.push(GlobalValue {
          value,
          mutability: global.mutability,
        });
      }
    }

    Store {
      funcs,
      memories,
      globals,
    }
  }

  pub fn get_instr(&self, func_idx: usize, pc: usize) -> Option<&Instructions> {
    match self.funcs.get(func_idx) {
      Some(f) => match f {
        FuncInstance::Internal(i) => match i.instrs.get(pc) {
          Some(i) => Some(i),
          None => None,
        },
        FuncInstance::External(_) => None,
      },
      None => panic!("func_idx out of range"),
    }
  }

  pub fn get_func(&self, func_idx: usize) -> &FuncInstance {
    match self.funcs.get(func_idx) {
      Some(f) => f,
      None => panic!("func_idx out of range"),
    }
  }

  pub fn call_func(&self, func_idx:usize, args: Vec<Value>) -> FuncInstance {
    let mut func_instance = self.get_func(func_idx).clone();
    match &mut func_instance {
      FuncInstance::Internal(func_instance) => {
        if args.len() == func_instance.param_types.len() {
          if args.iter().zip(&mut func_instance.param_types.iter()).all(|(a, b)| a.eq_for_value_type(b)) {
            for (i, a) in args.iter().enumerate() {
              func_instance.locals[i] = a.clone();
            }
          } else {
            panic!("Invalid args type");
          }
        } else {
          panic!("Invalid args length");
        }
      },
      FuncInstance::External(func_instance) => {
        func_instance.params = args;
      },
    }
    func_instance
  }

  pub fn call_func_by_name(&self, name: &str, args: Vec<Value>) -> FuncInstance {
    let func_idx = match self.funcs.iter()
      .position(|f| f.name().map_or(false, |n| n == name)){
      Some(idx) => idx,
      None => {
        println!("funcs: {:?}", self.funcs);
        panic!("function {} not found", name)
      },
    };
    self.call_func(func_idx, args)
  }
}

impl MemoryInst {
  pub fn store(&mut self, offset: u32, index: u32, size: u32, value: &[u8]) -> Result<()> {
    let addr = offset + index;
    let addr = addr as usize;
    let size = size as usize;
    if addr + size > self.memory.len() {
        return Err(anyhow!("Out of memory"));
    }
    self.memory[addr..addr + size].copy_from_slice(&value[0..size]);
    Ok(())
  }

  pub fn load(&self, offset: u32, index: u32, size: u32) -> Result<&[u8]> {
    let addr = offset + index;
    let addr = addr as usize;
    let size = size as usize;
    if addr + size > self.memory.len() {
        return Err(anyhow!("Out of memory"));
    }
    Ok(&self.memory[addr..addr + size])
  }
  pub fn size(&self) -> Value {
      Value::I32((self.memory.len() / PAGE_SIZE) as i32)
  }
  pub fn grow(&mut self, grow_size: usize) -> Value {
      let current_size = self.memory.len() / PAGE_SIZE;
      let new_size = current_size + grow_size;
      let max = self.max.unwrap_or(u32::MAX / PAGE_SIZE as u32);
      if new_size > max as usize {
          Value::I32(-1)
      } else {
          self.memory.resize(new_size * PAGE_SIZE, 0);
          Value::I32(current_size as i32)
      }
  }
  pub fn fill(&mut self, addr: usize, size: usize, value: u8) -> Result<()> {
      if addr + size > self.memory.len() {
          return Err(anyhow!("Out of memory"));
      }
      if size != 0 {
          self.memory[addr..addr + size].fill(value);
      }
      Ok(())
  }
  pub fn copy(&mut self, src: usize, dest: usize, size: usize) -> Result<()> {
      if src + size > self.memory.len() || dest + size > self.memory.len() {
          return Err(anyhow!("Out of memory"));
      }
      if size != 0 {
          let src_memory = self.memory[src..src + size].to_owned();
          self.memory[dest..dest + size].copy_from_slice(&src_memory);
      }
      Ok(())
  }
}