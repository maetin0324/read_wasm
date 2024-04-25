use nom::{
  bytes::complete::{take, take_till, tag},
  IResult
};
use nom_leb128::leb128_u32;
use super::instructions::Instructions;

pub struct Code {
  pub size: u32,
  // pub locals: Vec<LocalVar>,
  pub code: Vec<Instructions>,
}

pub struct LocalVar {
  pub count: u32,
  pub ty: u32,
}

impl Code {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Code>>{
    let (mut input, func_count) = leb128_u32(input)?;
    let mut codes: Vec<Code> = Vec::new();

    for _ in 0..func_count {
      let size: u32;
      let local_decls: u32;
      let instrs_bytes: &[u8];
      (input, size) = leb128_u32(input)?;
      (input, local_decls) = leb128_u32(input)?;
      (input, _) = take(local_decls*2)(input)?;
      (input, instrs_bytes) = take_till(|c| c == 0x0b)(input)?;
      (input, _) = tag([0x0b])(input)?;
      let (_, instrs) = Instructions::parse(instrs_bytes)?;


      codes.push(Code { size: size, code: instrs});
    }

    Ok((input, codes))
  }
}

