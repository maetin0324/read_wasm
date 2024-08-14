use nom::IResult;
use serde::{Deserialize, Serialize};

use crate::exec::value::Value;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

  pub fn parse_vec(input: &[u8], count: u32) -> IResult<&[u8], Vec<ValueType>> {
    let mut res = Vec::new();
    let mut input = input;
    for _ in 0..count {
      let value_type = ValueType::parse(input[0]);
      res.push(value_type);
      input = &input[1..];
    }
    Ok((input, res))
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