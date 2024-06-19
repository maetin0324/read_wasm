use super::value::Value;

#[derive(Debug, Clone)]
pub struct Frame {
  pub pc: isize,
  pub sp: usize,
  pub func_idx: usize,
  pub arity: usize,
  pub locals: Vec<Value>,
}

impl Frame {
  pub fn new(pc: isize, sp: usize, func_idx: usize, arity: usize, locals: Vec<Value>) -> Frame {
    Frame {
      pc,
      sp,
      func_idx,
      arity,
      locals,
    }
  }
}