use serde::{Deserialize, Serialize};

use crate::binary::{instructions::Instructions, wasm::Wasm};
use super::{func_instance::FuncInstance, value::Value};

pub const PAGE_SIZE: u32 = 65536; // 64Ki

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct Store {
  pub funcs: Vec<FuncInstance>,
  pub memories: Vec<MemoryInst>,
}

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct MemoryInst {
  pub memory: Vec<u8>,
  pub max: Option<u32>,
}

impl Store {
  pub fn new(funcs: Vec<FuncInstance>, wasm: &Wasm) -> Store {
    let mut memories = Vec::new();
    if let Some(ref memory_sec) = wasm.memory_section {
      for memory in memory_sec {
        let min = memory.min * PAGE_SIZE;
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

    Store {
      funcs,
      memories,
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