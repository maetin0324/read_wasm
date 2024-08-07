use core::panic;
use serde::{Deserialize, Serialize};

use crate::binary::import_sec::ImportDesc;
use crate::binary::instructions::Instructions;
use crate::binary::value_type::ValueType;
use crate::binary::wasm::Wasm;
use super::block_frame::BlockFrame;
use super::value::Value;

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub enum FuncInstance {
  External(ExternalFunc),
  Internal(InternalFunc),
}

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub struct InternalFunc {
  pub name: Option<String>,
  pub param_types: Vec<ValueType>,
  pub locals: Vec<Value>,
  pub instrs: Vec<Instructions>,
  pub pc: usize,
  pub label_stack: Vec<BlockFrame>,
}

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub struct ExternalFunc {
  pub env_name: String,
  pub name: Option<String>,
  pub param_types: Vec<ValueType>,
  pub return_types: Vec<ValueType>,
}

impl FuncInstance {
  pub fn new(wasm: &Wasm) -> Vec<FuncInstance> {
    let mut func_instances: Vec<FuncInstance> = Vec::new();

    match (&wasm.type_section, &wasm.function_section, &wasm.export_section, &wasm.code_section) {
      (Some(types), Some(funcs), Some(exports), Some(codes)) => {
        let import_func_count: usize;
        if let Some(inputs) = &wasm.import_section {
          import_func_count = inputs.len();
          for input in inputs.iter() {
            if let ImportDesc::Func(type_idx) = &input.desc {
              let (param_types, return_types) = match types.get(*type_idx as usize) {
                Some(t) => (t.param_types.clone(), t.return_types.clone()),
                None => panic!("type_idx {} not found", type_idx),
              };
              func_instances.push(FuncInstance::External(ExternalFunc {
                env_name: input.module.clone(),
                name: Some(input.field.clone()),
                param_types,
                return_types,
              }));
            }
          }
        } else { import_func_count = 0 }

        for (i, (func, code)) in funcs.iter().zip(codes.iter()).enumerate() {

          let param_types = match types.get(func.type_idx as usize) {
            Some(t) => t.param_types.clone(),
            None => panic!("type_idx {} not found", func.type_idx),
          };
          let mut local_types = param_types.clone();
          local_types.extend(code.locals.iter().map(|l| l.value_type.clone()).collect::<Vec<ValueType>>());
          let _ = code.locals.iter().map(|l| local_types.extend(l.to_value_type_vec()));

          let name: Option<String> = exports.iter().find_map(|e| {
            if e.func_idx == (i + import_func_count) as u32 {
              Some(e.name.clone())
            } else {
              None
            }
          });

          let mut locals = Vec::new();
          locals.extend(local_types.iter().map(|t| t.to_init_value()));
          

          func_instances.push(FuncInstance::Internal(InternalFunc {
            name,
            param_types,
            locals,
            instrs: code.instrs.clone(),
            pc: 0,
            label_stack: Vec::new(),
          }));
        
      }
    },
      _ => panic!("type_section, function_section, export_section, code_section is required"),
    }
    func_instances
  }

  pub fn name(&self) -> Option<&String> {
    match self {
      FuncInstance::Internal(i) => i.name.as_ref(),
      FuncInstance::External(e) => e.name.as_ref(),
    }
  }

  pub fn param_types(&self) -> Vec<ValueType> {
    match self {
      FuncInstance::Internal(i) => i.param_types.clone(),
      FuncInstance::External(e) => e.param_types.clone(),
    }
  }
}