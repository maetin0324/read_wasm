#[derive(Debug, PartialEq)]
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
}