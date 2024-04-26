use nom::{
  bytes::complete::{take, take_till, tag},
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
      let local_decls: u32;
      let mut local_bytes: &[u8];
      let instrs_bytes: &[u8];
      let mut locals: Vec<LocalVar> = Vec::new();

      (input, size) = leb128_u32(input)?;
      (input, local_decls) = leb128_u32(input)?;
      (input, local_bytes) = take(local_decls*2)(input)?;

      for _ in 0..local_decls {
        let count: u32;
        let types: &[u8];

        (local_bytes, count) = leb128_u32(local_bytes)?;
        (local_bytes, types) = take(1 as usize)(local_bytes)?;

        let value_type = ValueType::parse(types[0]);

        locals.push(LocalVar { count, value_type });
      }
      
      (input, instrs_bytes) = take_till(|c| c == 0x0b)(input)?;
      (input, _) = tag([0x0b])(input)?;

      let (_, instrs) = Instructions::parse(instrs_bytes)?;


      codes.push(Code { size: size, locals: locals, instrs: instrs });
    }

    Ok((input, codes))
  }
}

