use crate::binary::wasm::Wasm;
use crate::binary::instructions::{BlockType, Instructions};
use super::block_frame::BlockFrame;
use super::value::Value;
use super::func_instance::FuncInstance;

#[derive(Debug, Clone)]
pub struct ExecMachine {
  pub value_stack: Vec<Value>,
  pub call_stack: Vec<FuncInstance>,
  pub func_instances: Vec<FuncInstance>,
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
      func_instances: Vec::new(),
    }
  }

  pub fn exec(&mut self, wasm: Wasm, entry_point:&str, locals: Vec<Value>) -> Result<&ExecMachine, TrapError> {
    let func_instances = FuncInstance::new(wasm);
    self.call_stack.push(FuncInstance::call_by_name(entry_point, &func_instances, locals));
    self.func_instances = func_instances;
    self.run()
  }

  pub fn run(&mut self)  -> Result<&ExecMachine, TrapError> {
    while let Some(func) = self.call_stack.last_mut() {
      let Some(instr) = func.instrs.get(func.pc) else {
        self.call_stack.pop();
        continue;
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
            }
          };
          self.value_stack = frame.value_stack_evac;
          if let Some(v) = ret {
            self.value_stack.push(v);
          }
        },
        Instructions::Call(idx) => {
          let callee = match self.func_instances.get(*idx as usize) {
            Some(f) => f,
            None => {
              let message = format!("Call: function {} not found", idx);
              return Err(TrapError {
                message,
                vm: self.clone(),
              });
            }
          };
          let mut args = Vec::new();
          for _ in 0..callee.locals_len {
            match self.value_stack.pop() {
              Some(v) => args.insert(0, v),
              None => {
                return Err(TrapError {
                  message: "Call: value stack underflow".to_string(),
                  vm: self.clone(),
                });
              }
            }
          }
          let called_func = FuncInstance::call(*idx, &self.func_instances, args);
          func.pc += 1;
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
        Instructions::I32Add => {
          let a = self.value_stack.pop().unwrap();
          let b = self.value_stack.pop().unwrap();
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
        Instructions::I64Add => {
          let a = self.value_stack.pop().unwrap();
          let b = self.value_stack.pop().unwrap();
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
        Instructions::LocalGet(idx) => {
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
        _ => panic!("Unknown instruction: {:?}", instr),
      }
      func.pc += 1;
    }
    Ok(self)
  }
}