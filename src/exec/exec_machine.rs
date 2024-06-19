use crate::binary::wasm::Wasm;
use crate::binary::instructions::{BlockType, Instructions};
use super::block_frame::BlockFrame;
use super::store::Store;
use super::value::Value;
use super::func_instance::FuncInstance;

#[derive(Debug, Clone)]
pub struct ExecMachine {
  pub value_stack: Vec<Value>,
  pub call_stack: Vec<FuncInstance>,
  // pub func_instances: Vec<FuncInstance>,
  pub store: Store,
}

#[derive(Debug)]
pub struct TrapError {
  pub message: String,
  pub vm: ExecMachine,
}

impl Default for ExecMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecMachine {
  pub fn new() -> ExecMachine {
    ExecMachine {
      value_stack: Vec::new(),
      call_stack: Vec::new(),
      store: Store::default(),
    }
  }

  pub fn exec(&mut self, wasm: Wasm, entry_point:&str, locals: Vec<Value>) -> Result<&ExecMachine, TrapError> {
    let func_instances = FuncInstance::new(&wasm);
    let store = Store::new(func_instances.clone());
    self.call_stack.push(FuncInstance::call_by_name(entry_point, &func_instances, locals));
    // self.func_instances = func_instances;
    self.store = store;
    self.run()
  }

  pub fn run(&mut self)  -> Result<&ExecMachine, TrapError> {
    while let Some(mut func) = self.call_stack.pop() {
      let Some(instr) = func.instrs.get(func.pc) else {
        continue;
      };
      // println!("value_stack: {:?}", self.value_stack);
      // println!("call_stack: {:?}", self.call_stack);
      // println!("func: {:?}", func);
      // println!("instr: {:?}", instr);
      match instr {
        Instructions::Nop => {},
        Instructions::Unreachable => {
          println!{"call_stack: {:?}", self.call_stack};
          return Err(TrapError {
            message: "Unreachable".to_string(),
            vm: self.clone(),
          });
        },
        Instructions::Block(block) => {
          let frame = BlockFrame::new(self.value_stack.clone(), block.clone());
          self.value_stack.clear();
          func.label_stack.push(frame);
        },
        Instructions::End => {
          let frame = match func.label_stack.pop() {
            Some(f) => f,
            None => {
              return Err(TrapError {
                message: "End: label stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          
          self.end_block(frame)?;
        },
        Instructions::Br(idx) => {
          let block_depth = func.label_stack.len() - 1;
          if block_depth < *idx as usize {
            return Err(
              TrapError {
              message: "Br: label stack underflow".to_string(),
              vm: self.clone(),
            });
          }

          let mut end_count = block_depth - *idx as usize + 1;
          loop {
            func.pc += 1;
            let instrs = match func.instrs.get(func.pc) {
              Some(i) => i.clone(),
              None => {
                return Err(TrapError {
                  message: "Br: instruction not found".to_string(),
                  vm: self.clone(),
                });
              }
            };

            if instrs == Instructions::End {
              end_count -= 1;
              let frame = func.label_stack.pop().unwrap();
              self.end_block(frame)?;
              if end_count == 0 {
                break;
              }
            }
          }
        },
        Instructions::BrIf(idx) => {
          match self.value_stack.pop() {
            Some(Value::I32(val)) => {
              if val != 0 {
                let block_depth = func.label_stack.len() - 1;
                if block_depth < *idx as usize {
                  return Err(
                    TrapError {
                    message: "BrIf: label stack underflow".to_string(),
                    vm: self.clone(),
                  });
                }

                let mut end_count = block_depth - *idx as usize + 1;
                loop {
                  func.pc += 1;
                  let instrs = match func.instrs.get(func.pc) {
                    Some(i) => i.clone(),
                    None => {
                      return Err(TrapError {
                        message: "BrIf: instruction not found".to_string(),
                        vm: self.clone(),
                      });
                    }
                  };

                  if instrs == Instructions::End {
                    end_count -= 1;
                    let frame = func.label_stack.pop().unwrap();
                    self.end_block(frame)?;
                    if end_count == 0 {
                      break;
                    }
                  }
                }
              }
            },
            _ => {
              return Err(TrapError {
                message: "BrIf: invalid value type".to_string(),
                vm: self.clone(),
              });
            }
          }
        }
        Instructions::Return => {
          let frame = match func.label_stack.pop() {
            Some(f) => f,
            None => {
              return Err(TrapError {
                message: "Return: label stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.end_block(frame)?;
          continue;
        },
        Instructions::Call(idx) => {
          let callee = self.store.get_func(*idx as usize);
          let mut args = Vec::new();
          for pty in callee.param_types.iter() {
            match self.value_stack.pop() {
              Some(v) => {
                if v.eq_for_value_type(pty) {
                  args.insert(0, v);
                } else {
                  return Err(TrapError {
                    message: "Call: invalid value type".to_string(),
                    vm: self.clone(),
                  });
                }
              }
              None => {
                return Err(TrapError {
                  message: "Call: value stack underflow".to_string(),
                  vm: self.clone(),
                });
              }
            }
          }
          let called_func = FuncInstance::call(*idx, &self.store.funcs, args);
          self.call_stack.push(called_func);
          continue;
        }
        Instructions::Drop => {
          self.value_stack.pop();
        },
        Instructions::I32Const(val) => {
          self.value_stack.push(Value::I32(*val));
        },
        Instructions::I64Const(val) => {
          self.value_stack.push(Value::I64(*val));
        },
        Instructions::I32Eqz => {
          let val = match self.value_stack.pop() {
            Some(v) => v,
            None => {
              return Err(TrapError {
                message: "I32Eqz: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match val {
            Value::I32(v) => Value::I32(if v == 0 { 1 } else { 0 }),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I32Eqz".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        }
        Instructions::I32Eq => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I32Eq: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(if a == b { 1 } else { 0 }),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I32Eq".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::I64Eqz => {
          let val = match self.value_stack.pop() {
            Some(v) => v,
            None => {
              return Err(TrapError {
                message: "I64Eqz: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match val {
            Value::I64(v) => Value::I32(if v == 0 { 1 } else { 0 }),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I64Eqz".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        }
        Instructions::I64Eq => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I64Eq: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I32(if a == b { 1 } else { 0 }),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I64Eq".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::I32Add => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I32Add: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I32Add".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::I32Sub => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I32Sub: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(b - a),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I32Sub".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::I64Add => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I64Add: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a + b),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I64Add".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::I64Sub => {
          let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
              return Err(TrapError {
                message: "I64Sub: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          let ret = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(b - a),
            _ => {
              return Err(TrapError {
                message: "Invalid type for I64Sub".to_string(),
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(ret);
        },
        Instructions::LocalGet(idx) => {
          self.validate_local(&func, idx)?;
          let val = match func.locals.get(*idx as usize) {
            Some(v) => v.clone(),
            None => {
              let message = format!("LocalGet: local {} not found", idx);
              return Err(TrapError {
                message,
                vm: self.clone(),
              });
            }
          };
          self.value_stack.push(val);
        },
        Instructions::LocalSet(idx) => {
          self.validate_local(&func, idx)?;
          let val = match self.value_stack.pop() {
            Some(v) => v,
            None => {
              return Err(TrapError {
                message: "LocalSet: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          if !Value::match_value(&val, &func.locals[*idx as usize]) {
            return Err(TrapError {
              message: "LocalSet: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
          func.locals[*idx as usize] = val;
        },
        Instructions::LocalTee(idx) => {
          self.validate_local(&func, idx)?;
          let val = match self.value_stack.last() {
            Some(v) => v.clone(),
            None => {
              return Err(TrapError {
                message: "LocalTee: value stack underflow".to_string(),
                vm: self.clone(),
              });
            }
          };
          if !Value::match_value(&val, &func.locals[*idx as usize]) {
            return Err(TrapError {
              message: "LocalTee: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
          func.locals[*idx as usize] = val;
        },
        _ => panic!("Unknown instruction: {:?}", instr),
      }

      func.pc += 1;
      self.call_stack.push(func);
    }
    Ok(self)
  }

  pub fn end_block(&mut self, frame: BlockFrame) -> Result<(), TrapError> {
    let ret = match frame.return_type {
      BlockType::Void => None,
      BlockType::Value(t) => {
        match self.value_stack.pop() {
          Some(v) => {
            if v.eq_for_value_type(&t) {
              Some(v)
            } else {
              return Err(TrapError {
                message: "End: invalid return type".to_string(),
                vm: self.clone(),
              });
            }
          }
          None => {
            return Err(TrapError {
              message: "End: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        }
      },
    };
    self.value_stack = frame.value_stack_evac;
    if let Some(v) = ret {
      self.value_stack.push(v);
    }
    Ok(())
  }

  pub fn validate_local(&self, func: &FuncInstance, idx: &u32) -> Result<(), TrapError> {
    if func.locals.len() <= *idx as usize {
      return Err(TrapError {
        message: format!("LocalGet: local {} not found", idx),
        vm: self.clone(),
      });
    }
    Ok(())
  }
}