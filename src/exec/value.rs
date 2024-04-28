use crate::binary::value_type::ValueType;

#[derive(Debug, Clone, PartialEq)]
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
}