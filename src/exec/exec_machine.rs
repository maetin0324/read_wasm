use crate::binary::wasm::Wasm;
use crate::binary::instructions::Instructions;
use super::value::Value;

#[derive(Debug)]
pub struct ExecMachine {
  pub stack: Vec<Value>,
  pub locals: Vec<Value>,
  pub instrs: Vec<Instructions>,
  pub pc: usize,
}

impl ExecMachine {
  pub fn new() -> ExecMachine {
    ExecMachine {
      stack: Vec::new(),
      locals: Vec::new(),
      instrs: Vec::new(),
      pc: 0,
    }
  }

  pub fn exec(&mut self, wasm: &Wasm, entry_point:&str, locals: Vec<Value>) -> &ExecMachine {
    let entry_code = wasm.get_code_by_name(entry_point).unwrap();
    self.instrs.extend(entry_code.instrs.clone());
    self.locals.extend(locals);
    self.run()
  }

  pub fn run(&mut self)  -> &ExecMachine {
    loop {
      let Some(instr) = self.instrs.get(self.pc) else {
        break;
      
      };
      match instr {
        Instructions::I32Const(val) => {
          self.stack.push(Value::I32(*val));
        },
        Instructions::I64Const(val) => {
          self.stack.push(Value::I64(*val));
        },
        Instructions::I32Add => {
          let a = self.stack.pop().unwrap();
          let b = self.stack.pop().unwrap();
          let ret = match (a, b) {
            (Value::I32(a), Value::I32(b)) => Value::I32(a + b),
            _ => panic!("Invalid type for I32Add"),
          };
          self.stack.push(ret);
        },
        Instructions::I64Add => {
          let a = self.stack.pop().unwrap();
          let b = self.stack.pop().unwrap();
          let ret = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a + b),
            _ => panic!("Invalid type for I64Add"),
          };
          self.stack.push(ret);
        },
        Instructions::LocalGet(idx) => {
          let val = self.locals[*idx as usize].clone();
          self.stack.push(val);
        },
      }
      self.pc += 1;
      println!("machine: {:?}", self);
    }
    self
  }
}