use crate::binary::wasm::Wasm;
use crate::binary::instructions::Instructions;
use super::value::Value;
use super::func_instance::FuncInstance;

#[derive(Debug)]
pub struct ExecMachine {
  pub value_stack: Vec<Value>,
  pub call_stack: Vec<FuncInstance>,
  pub func_instances: Vec<FuncInstance>,
  pub locals: Vec<Value>,
  pub instrs: Vec<Instructions>,
  pub pc: usize,
}

impl ExecMachine {
  pub fn new() -> ExecMachine {
    ExecMachine {
      value_stack: Vec::new(),
      call_stack: Vec::new(),
      func_instances: Vec::new(),
      locals: Vec::new(),
      instrs: Vec::new(),
      pc: 0,
    }
  }

  pub fn exec(&mut self, wasm: Wasm, entry_point:&str, locals: Vec<Value>) -> &ExecMachine {
    let func_instances = FuncInstance::new(wasm);
    self.call_stack.push(FuncInstance::call_by_name(entry_point, &func_instances, locals));
    self.func_instances = func_instances;
    self.run()
  }

  pub fn run(&mut self)  -> &ExecMachine {
    loop {
      let func = match self.call_stack.last_mut() {
        Some(f) => f,
        None => break,
      };

      let Some(instr) = func.instrs.get(self.pc) else {
        self.call_stack.pop();
        continue;
      };

      println!("func: {:?}, instr: {:?}", func, instr);

      match instr {
        Instructions::Nop => {},
        Instructions::Unreachable => {
          println!{"call_stack: {:?}", self.call_stack};
          panic!("Unreachable");
        },
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
            _ => panic!("Invalid type for I32Add"),
          };
          self.value_stack.push(ret);
        },
        Instructions::I64Add => {
          let a = self.value_stack.pop().unwrap();
          let b = self.value_stack.pop().unwrap();
          let ret = match (a, b) {
            (Value::I64(a), Value::I64(b)) => Value::I64(a + b),
            _ => panic!("Invalid type for I64Add"),
          };
          self.value_stack.push(ret);
        },
        Instructions::LocalGet(idx) => {
          let val = match func.locals.get(*idx as usize) {
            Some(v) => v.clone(),
            None => panic!("LocalGet: idx {} not found", idx),
          };
          self.value_stack.push(val);
        },
        _ => panic!("Unknown instruction: {:?}", instr),
      }
      self.pc += 1;
    }
    self
  }
}