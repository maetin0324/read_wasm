use crate::binary::instructions::Instructions;
use super::value::Value;

pub fn exec_irelop(op: &Instructions, lhs: Value, rhs: Value) -> Result<Value, String> {
  match (lhs, rhs) {
    (Value::I32(lhs), Value::I32(rhs)) => Ok(exec_irelop_i32(op, lhs, rhs)),
    (Value::I64(lhs), Value::I64(rhs)) => Ok(exec_irelop_i64(op, lhs, rhs)),
    _ => Err("Invalid type for irelop".to_string()),
  }
}

fn exec_irelop_i32(op: &Instructions, lhs: i32, rhs: i32) -> Value {
  match op {
    Instructions::I32Eq => Value::I32((lhs == rhs) as i32),
    Instructions::I32Ne => Value::I32((lhs != rhs) as i32),
    Instructions::I32LtS => Value::I32((lhs < rhs) as i32),
    Instructions::I32LtU => Value::I32(((lhs as u32) < (rhs as u32)) as i32),
    Instructions::I32GtS => Value::I32((lhs > rhs) as i32),
    Instructions::I32GtU => Value::I32((lhs as u32 > rhs as u32) as i32),
    Instructions::I32LeS => Value::I32((lhs <= rhs) as i32),
    Instructions::I32LeU => Value::I32((lhs as u32 <= rhs as u32) as i32),
    Instructions::I32GeS => Value::I32((lhs >= rhs) as i32),
    Instructions::I32GeU => Value::I32((lhs as u32 >= rhs as u32) as i32),
    _ => unimplemented!(),
  }
}

fn exec_irelop_i64(op: &Instructions, lhs: i64, rhs: i64) -> Value {
  match op {
    Instructions::I64Eq => Value::I32((lhs == rhs) as i32),
    Instructions::I64Ne => Value::I32((lhs != rhs) as i32),
    Instructions::I64LtS => Value::I32((lhs < rhs) as i32),
    Instructions::I64LtU => Value::I32(((lhs as u64) < (rhs as u64)) as i32),
    Instructions::I64GtS => Value::I32((lhs > rhs) as i32),
    Instructions::I64GtU => Value::I32((lhs as u64 > rhs as u64) as i32),
    Instructions::I64LeS => Value::I32((lhs <= rhs) as i32),
    Instructions::I64LeU => Value::I32((lhs as u64 <= rhs as u64) as i32),
    Instructions::I64GeS => Value::I32((lhs >= rhs) as i32),
    Instructions::I64GeU => Value::I32((lhs as u64 >= rhs as u64) as i32),
    _ => unimplemented!(),
  }
}

pub fn exec_itestop(_op: &Instructions, v: Value) -> Result<Value, String> {
  match v {
    Value::I32(v) => Ok(Value::I32((v == 0) as i32)),
    Value::I64(v) => Ok(Value::I32((v == 0) as i32)),
    _ => Err("Invalid type for itestop".to_string()),
  }
}

pub fn exec_frelop(op: &Instructions, lhs: Value, rhs: Value) -> Result<Value, String> {
  match (lhs, rhs) {
    (Value::F32(lhs), Value::F32(rhs)) => Ok(exec_frelop_f32(op, lhs, rhs)),
    (Value::F64(lhs), Value::F64(rhs)) => Ok(exec_frelop_f64(op, lhs, rhs)),
    _ => Err("Invalid type for frelop".to_string()),
  }
}

fn exec_frelop_f32(op: &Instructions, lhs: f32, rhs: f32) -> Value {
  match op {
    Instructions::F32Eq => Value::I32((lhs == rhs) as i32),
    Instructions::F32Ne => Value::I32((lhs != rhs) as i32),
    Instructions::F32Lt => Value::I32((lhs < rhs) as i32),
    Instructions::F32Gt => Value::I32((lhs > rhs) as i32),
    Instructions::F32Le => Value::I32((lhs <= rhs) as i32),
    Instructions::F32Ge => Value::I32((lhs >= rhs) as i32),
    _ => unimplemented!(),
  }
}

fn exec_frelop_f64(op: &Instructions, lhs: f64, rhs: f64) -> Value {
  match op {
    Instructions::F64Eq => Value::I32((lhs == rhs) as i32),
    Instructions::F64Ne => Value::I32((lhs != rhs) as i32),
    Instructions::F64Lt => Value::I32((lhs < rhs) as i32),
    Instructions::F64Gt => Value::I32((lhs > rhs) as i32),
    Instructions::F64Le => Value::I32((lhs <= rhs) as i32),
    Instructions::F64Ge => Value::I32((lhs >= rhs) as i32),
    _ => unimplemented!(),
  }
}

pub fn exec_iuop(op: &Instructions, v: Value) -> Result<Value, String> {
  match v {
    Value::I32(v) => Ok(exec_iuop_i32(op, v)),
    Value::I64(v) => Ok(exec_iuop_i64(op, v)),
    _ => Err("Invalid type for iuop".to_string()),
  }
}

fn exec_iuop_i32(op: &Instructions, v: i32) -> Value {
  match op {
    Instructions::I32Clz => Value::I32(v.leading_zeros() as i32),
    Instructions::I32Ctz => Value::I32(v.trailing_zeros() as i32),
    Instructions::I32Popcnt => Value::I32(v.count_ones() as i32),
    Instructions::I32Eqz => Value::I32((v == 0) as i32),
    _ => unimplemented!(),
  }
}

fn exec_iuop_i64(op: &Instructions, v: i64) -> Value {
  match op {
    Instructions::I64Clz => Value::I64(v.leading_zeros() as i64),
    Instructions::I64Ctz => Value::I64(v.trailing_zeros() as i64),
    Instructions::I64Popcnt => Value::I64(v.count_ones() as i64),
    Instructions::I64Eqz => Value::I32((v == 0) as i32),
    _ => unimplemented!(),
  }
}

pub fn exec_ibinop(op: &Instructions, lhs: Value, rhs: Value) -> Result<Value, String> {
  match (lhs, rhs) {
    (Value::I32(lhs), Value::I32(rhs)) => Ok(exec_ibinop_i32(op, lhs, rhs)),
    (Value::I64(lhs), Value::I64(rhs)) => Ok(exec_ibinop_i64(op, lhs, rhs)),
    _ => Err("Invalid type for ibinop".to_string()),
  }
}

fn exec_ibinop_i32(op: &Instructions, lhs: i32, rhs: i32) -> Value {
  match op {
    Instructions::I32Add => Value::I32(lhs.wrapping_add(rhs)),
    Instructions::I32Sub => Value::I32(lhs.wrapping_sub(rhs)),
    Instructions::I32Mul => Value::I32(lhs.wrapping_mul(rhs)),
    Instructions::I32DivS => Value::I32(lhs.wrapping_div(rhs)),
    Instructions::I32DivU => Value::I32((lhs as u32).wrapping_div(rhs as u32) as i32),
    Instructions::I32RemS => Value::I32(lhs.wrapping_rem(rhs)),
    Instructions::I32RemU => Value::I32((lhs as u32).wrapping_rem(rhs as u32) as i32),
    Instructions::I32And => Value::I32(lhs & rhs),
    Instructions::I32Or => Value::I32(lhs | rhs),
    Instructions::I32Xor => Value::I32(lhs ^ rhs),
    Instructions::I32Shl => Value::I32(lhs.wrapping_shl(rhs as u32)),
    Instructions::I32ShrS => Value::I32(lhs.wrapping_shr(rhs as u32)),
    Instructions::I32ShrU => Value::I32((lhs as u32).wrapping_shr(rhs as u32) as i32),
    Instructions::I32Rotl => Value::I32(lhs.rotate_left(rhs as u32)),
    Instructions::I32Rotr => Value::I32(lhs.rotate_right(rhs as u32)),
    _ => unimplemented!(),
  }
}

fn exec_ibinop_i64(op: &Instructions, lhs: i64, rhs: i64) -> Value {
  match op {
    Instructions::I64Add => Value::I64(lhs.wrapping_add(rhs)),
    Instructions::I64Sub => Value::I64(lhs.wrapping_sub(rhs)),
    Instructions::I64Mul => Value::I64(lhs.wrapping_mul(rhs)),
    Instructions::I64DivS => Value::I64(lhs.wrapping_div(rhs)),
    Instructions::I64DivU => Value::I64((lhs as u64).wrapping_div(rhs as u64) as i64),
    Instructions::I64RemS => Value::I64(lhs.wrapping_rem(rhs)),
    Instructions::I64RemU => Value::I64((lhs as u64).wrapping_rem(rhs as u64) as i64),
    Instructions::I64And => Value::I64(lhs & rhs),
    Instructions::I64Or => Value::I64(lhs | rhs),
    Instructions::I64Xor => Value::I64(lhs ^ rhs),
    Instructions::I64Shl => Value::I64(lhs.wrapping_shl(rhs as u32)),
    Instructions::I64ShrS => Value::I64(lhs.wrapping_shr(rhs as u32)),
    Instructions::I64ShrU => Value::I64((lhs as u64).wrapping_shr(rhs as u32) as i64),
    Instructions::I64Rotl => Value::I64(lhs.rotate_left(rhs as u32)),
    Instructions::I64Rotr => Value::I64(lhs.rotate_right(rhs as u32)),
    _ => unimplemented!(),
  }
}