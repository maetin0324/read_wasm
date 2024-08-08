use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::binary::wasm::Wasm;
use crate::binary::instructions::{BlockType, Instructions};
use super::block_frame::BlockFrame;
use super::store::Store;
use super::value::Value;
use super::func_instance::{FuncInstance, InternalFunc};
use super::import::init_import;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecMachine {
  pub value_stack: Vec<Value>,
  pub call_stack: Vec<FuncInstance>,
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

  pub fn init(wasm: Wasm, entry_point:&str, locals: Vec<Value>) -> ExecMachine {
    let mut vm = ExecMachine::new();
    let func_instances = FuncInstance::new(&wasm);
    vm.store = Store::new(func_instances.clone(), &wasm);
    vm.call_stack.push(vm.store.call_func_by_name(entry_point, locals));
    vm
  }

  pub async fn deserialize(vm: &[u8]) -> Result<ExecMachine> {
    match bincode::deserialize(vm) {
      Ok(vm) => Ok(vm),
      Err(e) => Err(anyhow::anyhow!("Deserialize error: {:?}", e)),
    }
  }

  pub async fn exec(&mut self) -> Result<&ExecMachine, TrapError> {
    let mut import = init_import();
    while let Some(func) = self.call_stack.pop() {
      match func {
        FuncInstance::External(ext) => {
          match import.get_mut(&ext.env_name) {
            Some(h) => {
              match h.get_mut(&ext.name) {
                Some(func) => {
                  let ret = func(&mut self.store, ext.params);
                  self.value_stack.push(ret.unwrap().unwrap());
                },
                None => {
                  return Err(TrapError{
                    message: "unknown func name".to_string(),
                    vm: self.clone()
                  })
                }
              }
            }
            None => {
              return Err(TrapError{
                message: "unknown env name".to_string(),
                vm: self.clone()
              })
            }
          }
        },
        FuncInstance::Internal(func) => {self.run(func).await?;},
      }
    }
    Ok(self)
  }

  pub async fn run(&mut self, mut func: InternalFunc)  -> Result<&ExecMachine, TrapError> {
    let Some(instr) = func.instrs.get(func.pc) else {
      return Ok(self);
    };

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
        return Ok(self);
      },
      Instructions::Call(idx) => {
        // self.serialize_vm();
        let callee = self.store.get_func(*idx as usize);
        let mut args = Vec::new();
        for pty in callee.param_types().iter() {
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
        let called_func = self.store.call_func(*idx as usize, args);
        self.call_stack.push(called_func);
        return Ok(self);
      }
      Instructions::Drop => {
        self.value_stack.pop();
      },
      Instructions::I32Store { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };

        let addr = Into::<i32>::into(addr) as usize;
        let offset = (*offset) as usize;
        let at = addr + offset; // 2
        let end = at + size_of::<i32>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        let value: i32 = value.into();
        memory.memory[at..end].copy_from_slice(&value.to_le_bytes());
      },
      Instructions::I64Store { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };

        let addr = Into::<i32>::into(addr) as usize;
        let offset = (*offset) as usize;
        let at = addr + offset; // 2
        let end = at + size_of::<i64>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        let value: i64 = value.into();
        memory.memory[at..end].copy_from_slice(&value.to_le_bytes());
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
    self.call_stack.push(FuncInstance::Internal(func));
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

  pub fn validate_local(&self, func: &InternalFunc, idx: &u32) -> Result<(), TrapError> {
    if func.locals.len() <= *idx as usize {
      return Err(TrapError {
        message: format!("LocalGet: local {} not found", idx),
        vm: self.clone(),
      });
    }
    Ok(())
  }

  pub fn serialize_vm(&self) -> Vec<u8> {
    bincode::serialize(&self).unwrap()
  }
}