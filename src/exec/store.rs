use serde::{Deserialize, Serialize};

use crate::binary::instructions::Instructions;
use super::{func_instance::FuncInstance, value::Value};

#[derive(Debug, Default, Clone, PartialEq , Serialize, Deserialize)]
pub struct Store {
  pub funcs: Vec<FuncInstance>,
}

impl Store {
  pub fn new(funcs: Vec<FuncInstance>) -> Store {
    Store {
      funcs,
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