use core::panic;

use crate::binary::instructions::Instructions;
use crate::binary::value_type::ValueType;
use crate::binary::wasm::Wasm;
use super::block_frame::BlockFrame;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct FuncInstance {
  pub name: Option<String>,
  pub param_types: Vec<ValueType>,
  pub locals: Vec<Value>,
  pub instrs: Vec<Instructions>,
  pub pc: usize,
  pub label_stack: Vec<BlockFrame>,
}

impl FuncInstance {
  pub fn new(wasm: &Wasm) -> Vec<FuncInstance> {
    let mut func_instances: Vec<FuncInstance> = Vec::new();

    match (&wasm.type_section, &wasm.function_section, &wasm.export_section, &wasm.code_section) {
      (Some(types), Some(funcs), Some(exports), Some(codes)) => {
        for (i, (func, code)) in funcs.iter().zip(codes.iter()).enumerate() {

          let param_types = match types.get(func.type_idx as usize) {
            Some(t) => t.param_types.clone(),
            None => panic!("type_idx {} not found", func.type_idx),
          };
          let mut local_types = param_types.clone();
          local_types.extend(code.locals.iter().map(|l| l.value_type.clone()).collect::<Vec<ValueType>>());
          let _ = code.locals.iter().map(|l| local_types.extend(l.to_value_type_vec()));

          let name: Option<String> = exports.iter().find_map(|e| {
            if e.func_idx == i as u32 {
              Some(e.name.clone())
            } else {
              None
            }
          });

          let mut locals = Vec::new();
          locals.extend(local_types.iter().map(|t| t.to_init_value()));
          

          func_instances.push(FuncInstance {
            name,
            param_types,
            locals,
            instrs: code.instrs.clone(),
            pc: 0,
            label_stack: Vec::new(),
          });
        }
      },
      _ => panic!("type_section, function_section, export_section, code_section is required"),
    }
    func_instances
  }

  pub fn call(func_idx: u32, func_instances: &[FuncInstance], args: Vec<Value>) -> FuncInstance {
    let mut func_instance = func_instances[func_idx as usize].clone();
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
    func_instance
  }

  pub fn call_by_name(name: &str, func_instances: &[FuncInstance], args: Vec<Value>) -> FuncInstance {
    let func_idx = match func_instances.iter()
      .position(|f| f.name.as_ref().map_or(false, |n| n == name)){
      Some(idx) => idx,
      None => panic!("function {} not found", name),
    } as u32;
    FuncInstance::call(func_idx, func_instances, args)
  }
}