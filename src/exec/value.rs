use serde::{Deserialize, Serialize};
use crate::binary::value_type::ValueType;

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
}

impl Value {
  pub fn eq_for_value_type(&self, other: &ValueType) -> bool {
    match (self, other) {
      (Value::I32(_), ValueType::I32) => true,
      (Value::I64(_), ValueType::I64) => true,
      (Value::F32(_), ValueType::F32) => true,
      (Value::F64(_), ValueType::F64) => true,
      _ => false,
    }
  }

  pub fn init_from_valtype(valtype: &ValueType) -> Value {
    match valtype {
      ValueType::I32 => Value::I32(0),
      ValueType::I64 => Value::I64(0),
      ValueType::F32 => Value::F32(0.0),
      ValueType::F64 => Value::F64(0.0),
    }
  }

  pub fn match_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
      (&Value::I32(_), &Value::I32(_)) => true,
      (&Value::I64(_), &Value::I64(_)) => true,
      (&Value::F32(_), &Value::F32(_)) => true,
      (&Value::F64(_), &Value::F64(_)) => true,
      _ => false,
    }
  }

  pub fn parse_from_i64_vec(input: Vec<i64>) -> Vec<Value> {
    input.iter().map(|&x| Value::I64(x)).collect()
  }
}