use serde::{Deserialize, Serialize};
use crate::binary::value_type::ValueType;

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
}

impl Default for Value {
  fn default() -> Self {
    Value::I32(0)
  }
}

impl Value {
  pub fn eq_for_value_type(&self, other: &ValueType) -> bool {
    matches!((self, other), 
        (Value::I32(_), ValueType::I32) 
      | (Value::I64(_), ValueType::I64) 
      | (Value::F32(_), ValueType::F32) 
      | (Value::F64(_), ValueType::F64)
    )
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
    matches!((a, b), 
        (&Value::I32(_), &Value::I32(_)) 
      | (&Value::I64(_), &Value::I64(_)) 
      | (&Value::F32(_), &Value::F32(_)) 
      | (&Value::F64(_), &Value::F64(_))
    )
  }

  pub fn parse_from_i64_vec(input: Vec<i64>) -> Vec<Value> {
    input.iter().map(|&x| Value::I64(x)).collect()
  }
}

impl From<Value> for i32 {
  fn from(value: Value) -> Self {
    match value {
        Value::I32(value) => value,
        _ => panic!("type mismatch"),
    }
  }
}

impl From<Value> for i64 {
  fn from(value: Value) -> Self {
    match value {
        Value::I64(value) => value,
        _ => panic!("type mismatch"),
    }
  }
}

impl From<Value> for f32 {
  fn from(value: Value) -> Self {
    match value {
        Value::F32(value) => value,
        _ => panic!("type mismatch"),
    }
  }
} 

impl From<Value> for f64 {
  fn from(value: Value) -> Self {
    match value {
        Value::F64(value) => value,
        _ => panic!("type mismatch"),
    }
  }
}

impl From<i32> for Value {
  fn from(value: i32) -> Self {
    Value::I32(value)
  }
}

impl From<i64> for Value {
  fn from(value: i64) -> Self {
    Value::I64(value)
  }
}

impl From<f32> for Value {
  fn from(value: f32) -> Self {
    Value::F32(value)
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    Value::F64(value)
  }
}