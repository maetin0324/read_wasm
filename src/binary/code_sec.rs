use nom::{
  bytes::complete::take,
  IResult
};
use nom_leb128::leb128_u32;
use super::instructions::Instructions;
use super::value_type::ValueType;

#[derive(Debug, PartialEq)]
pub struct Code {
  pub size: u32,
  pub locals: Vec<LocalVar>,
  pub instrs: Vec<Instructions>,
}

#[derive(Debug, PartialEq)]
pub struct LocalVar {
  pub count: u32,
  pub value_type: ValueType,
}

impl Code {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Code>>{
    let (mut input, func_count) = leb128_u32(input)?;
    let mut codes: Vec<Code> = Vec::new();

    for _ in 0..func_count {
      let size: u32;
      let mut code_body: &[u8];
      let local_decls: u32;
      let mut local_bytes: &[u8];
      let mut locals: Vec<LocalVar> = Vec::new();

      (input, size) = leb128_u32(input)?;

      (input, code_body) = take(size as usize)(input)?;

      (code_body, local_decls) = leb128_u32(code_body)?;
      (code_body, local_bytes) = take(local_decls*2)(code_body)?;

      for _ in 0..local_decls {
        let count: u32;
        let types: &[u8];

        (local_bytes, count) = leb128_u32(local_bytes)?;
        (local_bytes, types) = take(1_usize)(local_bytes)?;

        let value_type = ValueType::parse(types[0]);

        locals.push(LocalVar { count, value_type });
      }
      
      // (input, instrs_bytes) = take_till(|c| c == 0x0b)(input)?;
      // (input, _) = tag([0x0b])(input)?;
      // (input, instrs_bytes) = take(size as usize)(input)?;

      let (_, mut instrs) = Instructions::parse(code_body)?;

      match instrs.pop() {
        Some(Instructions::End) => {},
        _ => panic!("Code must end with End instruction"),
      }


      codes.push(Code { size, locals, instrs });
    }

    Ok((input, codes))
  }
}

impl LocalVar {
  pub fn to_value_type_vec(&self) -> Vec<ValueType> {
    let mut vec = Vec::new();
    for _ in 0..self.count {
      vec.push(self.value_type.clone());
    }
    vec
  }
}

