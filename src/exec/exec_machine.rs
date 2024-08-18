use core::panic;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::binary::value_type::ValueType;
use crate::binary::wasm::Wasm;
use crate::binary::instructions::{BlockType, Instructions};
use super::block_frame::BlockFrame;
use super::store::Store;
use super::value::Value;
use super::func_instance::{FuncInstance, InternalFunc};
use super::import::init_import;
use super::wasi::WasiSnapshotPreview1;

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

  pub async fn exec(&mut self, wasi: &mut WasiSnapshotPreview1) -> Result<&ExecMachine, TrapError> {
    let mut import = init_import();
    while let Some(func) = self.call_stack.pop() {
      match func {
        FuncInstance::External(ext) => {
          match import.get_mut(&ext.env_name) {
            Some(h) => {
              match h.get_mut(&ext.name) {
                Some(func) => {
                  let ret = func(wasi, &mut self.store, ext.params);
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

  pub fn init_without_start(wasm: Wasm) -> ExecMachine {
    let mut vm = ExecMachine::new();
    let func_instances = FuncInstance::new(&wasm);
    vm.store = Store::new(func_instances.clone(), &wasm);
    vm
  }

  pub async fn invoke(&mut self,wasi: &mut WasiSnapshotPreview1, entry_point: String, locals: Vec<Value>) -> Result<&ExecMachine, TrapError> {
    self.call_stack.push(self.store.call_func_by_name(&entry_point, locals));
    self.exec(wasi).await
  }

  pub async fn run(&mut self, mut func: InternalFunc)  -> Result<&ExecMachine, TrapError> {
    let Some(instr) = func.instrs.get(func.pc) else {
      return Ok(self);
    };

    println!("call_stack: {}", self.call_stack.len());
    // if self.call_stack.len() == 6 {

    println!("instr: {:?}, pc: {}, stack: {:?}, locals: {:?}", instr, func.pc, self.value_stack, func.locals);
    println!("label_stack: {:#?}", func.label_stack);
    // }
    match instr {
      Instructions::Nop => {},
      Instructions::Unreachable => {
        return Err(TrapError {
          message: "Unreachable".to_string(),
          vm: self.clone(),
        });
      },
      Instructions::Block(block) => {
        let frame = BlockFrame::new(self.value_stack.clone(), block.clone(), false);
        self.value_stack.clear();
        func.label_stack.push(frame);
      },
      Instructions::Loop(block) => {
        let frame = BlockFrame::new(self.value_stack.clone(), block.clone(), true);
        self.value_stack.clear();
        func.label_stack.push(frame);
      },
      Instructions::If(block) => {
        let val = self.value_stack.pop().unwrap();
        match val {
          Value::I32(v) => {
            if v != 0 {
              let frame = BlockFrame::new(self.value_stack.clone(), block.clone(), false);
              self.value_stack.clear();
              func.label_stack.push(frame);
            } else {
              let frame = BlockFrame::new(self.value_stack.clone(), block.clone(), false);
              self.value_stack.clear();
              func.label_stack.push(frame);
              func.pc = block.jump_pc;
            }
          },
          _ => {
            return Err(TrapError {
              message: "If: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::Else => {
        loop {
          func.pc += 1;
          let instr = func.instrs.get(func.pc).unwrap();
          if instr == &Instructions::End {
            func.label_stack.pop();
            break;
          }
        }
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
        let idx = *idx as usize;
        self.pop_labels(&mut func, idx)?;
      },
      Instructions::BrIf(idx) => {
        match self.value_stack.pop() {
          Some(Value::I32(val)) => {
            if val != 0 {
              let idx = *idx as usize;
              self.pop_labels(&mut func, idx)?;
            }
          },
          _ => {
            return Err(TrapError {
              message: "BrIf: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::BrTable(labelidxs, default) => {
        let val = match self.value_stack.pop() {
          Some(Value::I32(v)) => v,
          _ => {
            return Err(TrapError {
              message: "BrTable: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
        };

        let idx = if val as usize >= labelidxs.len() {
          *default
        } else {
          labelidxs[val as usize]
        };
        
        self.pop_labels(&mut func, idx as usize)?;
      },
      Instructions::Return => {
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
        func.pc += 1;
        self.call_stack.push(FuncInstance::Internal(func));
        self.call_stack.push(called_func);
        return Ok(self);
      }
      Instructions::Drop => {
        self.value_stack.pop();
      },
      Instructions::Select => {
        let (Some(c), Some(a), Some(b)) = (self.value_stack.pop(), self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "Select: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        if c.eq_for_value_type(&ValueType::I32) {
          if c != Value::I32(0) {
            self.value_stack.push(a);
          } else {
            self.value_stack.push(b);
          }
        } else {
          return Err(TrapError {
            message: "Select: invalid value type".to_string(),
            vm: self.clone(),
          });
        }
      },
      Instructions::SelectValtype(_) => {
        let (Some(c), Some(a), Some(b)) = (self.value_stack.pop(), self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "Select: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        if c.eq_for_value_type(&ValueType::I32) {
          if c != Value::I32(0) {
            self.value_stack.push(a);
          } else {
            self.value_stack.push(b);
          }
        } else {
          return Err(TrapError {
            message: "Select: invalid value type".to_string(),
            vm: self.clone(),
          });
        }
      }
      Instructions::I32Load { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I32Load: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i32>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i32 = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        self.value_stack.push(Value::I32(value));
      },
      Instructions::I64Load { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i64>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i64 = i64::from_le_bytes([value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7]]);
        self.value_stack.push(Value::I64(value));
      },
      Instructions::F32Load { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "F32Load: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<f32>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: f32 = f32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        self.value_stack.push(Value::F32(value));
      },
      Instructions::F64Load { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "F64Load: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<f64>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: f64 = f64::from_le_bytes([value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7]]);
        self.value_stack.push(Value::F64(value));
      },
      Instructions::I32Load8S { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I32Load8S: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i8>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i8 = i8::from_le_bytes([value[0]]);
        self.value_stack.push(Value::I32(value as i32));
      },
      Instructions::I32Load8U { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I32Load8U: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<u8>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: u8 = u8::from_le_bytes([value[0]]);
        self.value_stack.push(Value::I32(value as i32));
      },
      Instructions::I32Load16S { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I32Load16S: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i16>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i16 = i16::from_le_bytes([value[0], value[1]]);
        self.value_stack.push(Value::I32(value as i32));
      },
      Instructions::I32Load16U { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I32Load16U: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<u16>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: u16 = u16::from_le_bytes([value[0], value[1]]);
        self.value_stack.push(Value::I32(value as i32));
      },
      Instructions::I64Load8S { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load8S: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i8>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i8 = i8::from_le_bytes([value[0]]);
        self.value_stack.push(Value::I64(value as i64));
      },
      Instructions::I64Load8U { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load8U: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<u8>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: u8 = u8::from_le_bytes([value[0]]);
        self.value_stack.push(Value::I64(value as i64));
      },
      Instructions::I64Load16S { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load16S: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i16>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i16 = i16::from_le_bytes([value[0], value[1]]);
        self.value_stack.push(Value::I64(value as i64));
      },
      Instructions::I64Load16U { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load16U: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<u16>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: u16 = u16::from_le_bytes([value[0], value[1]]);
        self.value_stack.push(Value::I64(value as i64));
      },
      Instructions::I64Load32S { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load32S: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i32>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: i32 = i32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        self.value_stack.push(Value::I64(value as i64));
      },
      Instructions::I64Load32U { align: _, offset } => {
        let Some(addr) = self.value_stack.pop() 
        else { 
          return Err(TrapError {
            message: "I64Load32U: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<u32>();
        let value = self.store.memories[0].load(*offset, addr, size as u32).unwrap();
        let value: u32 = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        self.value_stack.push(Value::I64(value as i64));
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
        let end = at + std::mem::size_of::<i32>();

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
        let end = at + std::mem::size_of::<i64>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        let value: i64 = value.into();
        memory.memory[at..end].copy_from_slice(&value.to_le_bytes());
      },
      Instructions::F32Store { align: _, offset } => {
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
        let end = at + std::mem::size_of::<f32>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        let value: f32 = value.into();
        memory.memory[at..end].copy_from_slice(&value.to_le_bytes());
      },
      Instructions::F64Store { align: _, offset } => {
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
        let end = at + std::mem::size_of::<f64>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        let value: f64 = value.into();
        memory.memory[at..end].copy_from_slice(&value.to_le_bytes());
      },
      Instructions::I32Store8 { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let value: i32 = value.into();
        let value = value.to_le_bytes().to_vec();
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i8>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        match memory.store(*offset, addr, size as u32, &value) {
          Ok(_) => {},
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Store8: {}", e),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::I32Store16 { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let value: i32 = value.into();
        let value = value.to_le_bytes().to_vec();
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i16>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        match memory.store(*offset, addr, size as u32, &value) {
          Ok(_) => {},
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Store16: {}", e),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::I64Store8 { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let value: i64 = value.into();
        let value = value.to_le_bytes().to_vec();
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i8>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        match memory.store(*offset, addr, size as u32, &value) {
          Ok(_) => {},
          Err(e) => {
            return Err(TrapError {
              message: format!("I64Store8: {}", e),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::I64Store16 { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let value: i64 = value.into();
        let value = value.to_le_bytes().to_vec();
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i16>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        match memory.store(*offset, addr, size as u32, &value) {
          Ok(_) => {},
          Err(e) => {
            return Err(TrapError {
              message: format!("I64Store16: {}", e),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::I64Store32 { align: _, offset } => {
        let (Some(value), Some(addr)) = (self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "I32Store: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        let value: i64 = value.into();
        let value = value.to_le_bytes().to_vec();
        let addr = Into::<i32>::into(addr) as u32;
        let size = std::mem::size_of::<i32>();

        let memory = self
                        .store
                        .memories
                        .get_mut(0)
                        .unwrap();
        match memory.store(*offset, addr, size as u32, &value) {
          Ok(_) => {},
          Err(e) => {
            return Err(TrapError {
              message: format!("I64Store32: {}", e),
              vm: self.clone(),
            });
          }
        }
      },
      Instructions::MemorySize => {
        let size = self.store.memories[0].size();
        self.value_stack.push(size);
      },
      Instructions::MemoryCopy => {
        let (Some(dst), Some(src), Some(len)) = (self.value_stack.pop(), self.value_stack.pop(), self.value_stack.pop()) 
        else { 
          return Err(TrapError {
            message: "MemoryCopy: value stack underflow".to_string(), 
            vm: self.clone() 
          })
        };
        match (dst, src, len) {
          (Value::I32(dst), Value::I32(src), Value::I32(len)) => {
            self.store.memories[0].copy(dst as usize, src as usize, len as usize).unwrap();
          },
          _ => {
            return Err(TrapError {
              message: "MemoryCopy: invalid value type".to_string(),
              vm: self.clone(),
            });
          }
        }
      }
      Instructions::I32Const(val) => {
        self.value_stack.push(Value::I32(*val));
      },
      Instructions::I64Const(val) => {
        self.value_stack.push(Value::I64(*val));
      },
      Instructions::F32Const(val) => {
        self.value_stack.push(Value::F32(*val));
      },
      Instructions::F64Const(val) => {
        self.value_stack.push(Value::F64(*val));
      },
      Instructions::I32Eqz
      | Instructions::I64Eqz => {
        let val = self.value_stack.pop().unwrap();
        let ret = match crate::exec::op::exec_itestop(instr, val) {
          Ok(v) => v,
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Eqz: {}", e),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      }
      Instructions::I32Eq 
      | Instructions::I32Ne
      | Instructions::I32LtS
      | Instructions::I32LtU
      | Instructions::I32GtS
      | Instructions::I32GtU
      | Instructions::I32LeS
      | Instructions::I32LeU
      | Instructions::I32GeS
      | Instructions::I32GeU
      | Instructions::I64Eq
      | Instructions::I64Ne
      | Instructions::I64LtS
      | Instructions::I64LtU
      | Instructions::I64GtS
      | Instructions::I64GtU
      | Instructions::I64LeS
      | Instructions::I64LeU
      | Instructions::I64GeS
      | Instructions::I64GeU => {
        let (b, a) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(b), Some(a)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "I32Eq: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match crate::exec::op::exec_irelop(instr, b, a) {
          Ok(v) => v,
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Eq: {}", e),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Eq
      | Instructions::F32Ne
      | Instructions::F32Lt
      | Instructions::F32Gt
      | Instructions::F32Le
      | Instructions::F32Ge
      | Instructions::F64Eq
      | Instructions::F64Ne
      | Instructions::F64Lt
      | Instructions::F64Gt
      | Instructions::F64Le
      | Instructions::F64Ge => {
        let (b, a) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(b), Some(a)) => (b, a),
          _ => {
            return Err(TrapError {
              message: "F32Eq: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match crate::exec::op::exec_frelop(instr, b, a) {
          Ok(v) => v,
          Err(e) => {
            return Err(TrapError {
              message: format!("F32Eq: {}", e),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32Clz
      | Instructions::I32Ctz
      | Instructions::I32Popcnt
      | Instructions::I64Clz
      | Instructions::I64Ctz
      | Instructions::I64Popcnt => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32Ctz: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match crate::exec::op::exec_iuop(instr, val) {
          Ok(v) => v,
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Ctz: {}", e),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32Add
      | Instructions::I32Sub
      | Instructions::I32Mul
      | Instructions::I32DivS
      | Instructions::I32DivU
      | Instructions::I32RemS
      | Instructions::I32RemU
      | Instructions::I32And
      | Instructions::I32Or
      | Instructions::I32Xor
      | Instructions::I32Shl
      | Instructions::I32ShrS
      | Instructions::I32ShrU
      | Instructions::I32Rotl
      | Instructions::I32Rotr
      | Instructions::I64Add
      | Instructions::I64Sub
      | Instructions::I64Mul
      | Instructions::I64DivS
      | Instructions::I64DivU
      | Instructions::I64RemS
      | Instructions::I64RemU
      | Instructions::I64And
      | Instructions::I64Or
      | Instructions::I64Xor
      | Instructions::I64Shl
      | Instructions::I64ShrS
      | Instructions::I64ShrU
      | Instructions::I64Rotl
      | Instructions::I64Rotr => {
        let (b, a) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(b), Some(a)) => (b, a),
          _ => {
            return Err(TrapError {
              message: "I32Add: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match crate::exec::op::exec_ibinop(instr, a, b) {
          Ok(v) => v,
          Err(e) => {
            return Err(TrapError {
              message: format!("I32Add: {}", e),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Abs => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Abs: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.abs()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Abs".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Neg => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Neg: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(-v),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Neg".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Ceil => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Ceil: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.ceil()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Ceil".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Floor => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Floor: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.floor()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Floor".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Trunc => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Trunc: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.trunc()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Trunc".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Nearest => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Nearest: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.round()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Nearest".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Sqrt => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32Sqrt: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::F32(v.sqrt()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Sqrt".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Add => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Add: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(a + b),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Add".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Sub => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Sub: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(b - a),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Sub".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Mul => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Mul: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(a * b),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Mul".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Div => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Div: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(0.0), Value::F32(_)) => Value::F32(0.0),
          (Value::F32(a), Value::F32(b)) => Value::F32(b / a),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Div".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Min => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Min: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(a.min(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Min".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Max => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Max: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(a.max(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Max".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32Copysign => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F32Copysign: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F32(a), Value::F32(b)) => Value::F32(a.copysign(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32Copysign".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Abs => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Abs: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.abs()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Abs".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Neg => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Neg: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(-v),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Neg".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Ceil => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Ceil: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.ceil()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Ceil".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Floor => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Floor: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.floor()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Floor".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Trunc => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Trunc: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.trunc()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Trunc".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Nearest => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Nearest: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.round()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Nearest".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Sqrt => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64Sqrt: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::F64(v.sqrt()),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Sqrt".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Add => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Add: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(a + b),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Add".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Sub => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Sub: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(b - a),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Sub".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Mul => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Mul: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(a * b),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Mul".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Div => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Div: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(0.0), Value::F64(_)) => Value::F64(0.0),
          (Value::F64(a), Value::F64(b)) => Value::F64(b / a),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Div".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Min => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Min: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(a.min(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Min".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Max => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Max: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(a.max(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Max".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64Copysign => {
        let (a, b) = match (self.value_stack.pop(), self.value_stack.pop()) {
          (Some(a), Some(b)) => (a, b),
          _ => {
            return Err(TrapError {
              message: "F64Copysign: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match (a, b) {
          (Value::F64(a), Value::F64(b)) => Value::F64(a.copysign(b)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64Copysign".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32WrapI64 => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32WrapI64: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::I32(v as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32WrapI64".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32TruncF32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32TruncF32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::I32(v as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32TruncF32S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32TruncF32U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32TruncF32U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::I32(v as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32TruncF32U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32TruncF64S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32TruncF64S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::I32(v as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32TruncF64S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32TruncF64U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32TruncF64U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::I32(v as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32TruncF64U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64ExtendI32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64ExtendI32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64ExtendI32S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64ExtendI32U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64ExtendI32U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64ExtendI32U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64TruncF32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64TruncF32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64TruncF32S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64TruncF32U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64TruncF32U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64TruncF32U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64TruncF64S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64TruncF64S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64TruncF64S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64TruncF64U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64TruncF64U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::I64(v as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64TruncF64U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32ConvertI32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32ConvertI32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::F32(v as f32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32ConvertI32S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32ConvertI32U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32ConvertI32U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::F32(v as f32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32ConvertI32U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32ConvertI64S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32ConvertI64S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::F32(v as f32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32ConvertI64S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32ConvertI64U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32ConvertI64U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::F32(v as f32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32ConvertI64U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64ConvertI32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64ConvertI32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::F64(v as f64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64ConvertI32S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64ConvertI32U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64ConvertI32U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::F64(v as f64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64ConvertI32U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64ConvertI64S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64ConvertI64S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::F64(v as f64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64ConvertI64S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64ConvertI64U => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64ConvertI64U: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::F64(v as f64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64ConvertI64U".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32ReinterpretF32 => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32ReinterpretF32: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F32(v) => Value::I32(v.to_bits() as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32ReinterpretF32".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64ReinterpretF64 => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64ReinterpretF64: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::F64(v) => Value::I64(v.to_bits() as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64ReinterpretF64".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F32ReinterpretI32 => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F32ReinterpretI32: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::F32(f32::from_bits(v as u32)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F32ReinterpretI32".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::F64ReinterpretI64 => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "F64ReinterpretI64: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::F64(f64::from_bits(v as u64)),
          _ => {
            return Err(TrapError {
              message: "Invalid type for F64ReinterpretI64".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32Extend8S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32Extend8S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::I32((v as i8) as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32Extend8S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I32Extend16S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I32Extend16S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I32(v) => Value::I32((v as i16) as i32),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I32Extend16S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64Extend8S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64Extend8S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::I64((v as i8) as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64Extend8S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64Extend16S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64Extend16S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::I64((v as i16) as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64Extend16S".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(ret);
      },
      Instructions::I64Extend32S => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "I64Extend32S: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        let ret = match val {
          Value::I64(v) => Value::I64((v as i32) as i64),
          _ => {
            return Err(TrapError {
              message: "Invalid type for I64Extend32S".to_string(),
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
      Instructions::GlobalGet(idx) => {
        let val = match self.store.globals.get(*idx as usize) {
          Some(v) => v.value.clone(),
          None => {
            return Err(TrapError {
              message: "GlobalGet: global not found".to_string(),
              vm: self.clone(),
            });
          }
        };
        self.value_stack.push(val);
      },
      Instructions::GlobalSet(idx) => {
        let val = match self.value_stack.pop() {
          Some(v) => v,
          None => {
            return Err(TrapError {
              message: "GlobalSet: value stack underflow".to_string(),
              vm: self.clone(),
            });
          }
        };
        
        let global = match self.store.globals.get_mut(*idx as usize) {
          Some(g) => g,
          None => {
            return Err(TrapError {
              message: "GlobalSet: global not found".to_string(),
              vm: self.clone(),
            });
          }
        };

        if !global.mutability {
          return Err(TrapError {
            message: "GlobalSet: global is immutable".to_string(),
            vm: self.clone(),
          });
        };

        if !Value::match_value(&val, &global.value) {
          return Err(TrapError {
            message: "GlobalSet: invalid value type".to_string(),
            vm: self.clone(),
          });
        }
        global.value = val;
      }
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

  pub fn pop_labels(&mut self, func: &mut InternalFunc, count: usize) -> Result<(), TrapError> {
    for _ in 0..count {
      match func.label_stack.last() {
        Some(_) => {
            func.label_stack.pop();
        },
        None => {
          return Err(TrapError {
            message: "End: label stack underflow".to_string(),
            vm: self.clone(),
          });
        }
      }
    }


    match func.label_stack.pop() {
      Some(frame) => {
        func.pc = frame.jump_pc;
        if frame.is_loop {
          func.label_stack.push(frame);
        } else {
          self.end_block(frame)?;
        }
      }
      None => {
        return Err(TrapError {
          message: "End: label stack underflow".to_string(),
          vm: self.clone(),
        });
      }
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