use crate::binary::instructions::Instructions;
use super::func_instance::FuncInstance;

#[derive(Debug, Default, Clone)]
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
      Some(f) => match f.instrs.get(pc) {
        Some(i) => Some(i),
        None => None,
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
}