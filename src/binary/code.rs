use nom::{
  IResult,
  // bytes::complete::tag, 
  bytes::complete::take
};
use nom_leb128::leb128_u32;

pub struct Code {
  pub size: u32,
  // pub locals: Vec<LocalVar>,
  // code: Vec<u8>,
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
      let instrs: &[u8];
      (input, size) = leb128_u32(input)?;
      (input, local_decls) = leb128_u32(input)?;
      (input, _) = take(local_decls)(input)?;
      (input, instrs) = take(size - local_decls)(input)?;

      codes.push(Code { size });
    }

    Ok((input, codes))
  }
}

