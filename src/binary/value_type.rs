use crate::exec::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
  I32,
  I64,
  F32,
  F64,
}

impl ValueType {
  pub fn parse(input: u8) -> ValueType {
    match input {
      0x7F => ValueType::I32,
      0x7E => ValueType::I64,
      0x7D => ValueType::F32,
      0x7C => ValueType::F64,
      _ => panic!("Unknown value type: {:#x?}", input),
    }
  }

  pub fn to_init_value(&self) -> Value {
    match self {
      ValueType::I32 => Value::I32(0),
      ValueType::I64 => Value::I64(0),
      ValueType::F32 => Value::F32(0.0),
      ValueType::F64 => Value::F64(0.0),
    }
  }
}