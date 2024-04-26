#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
}