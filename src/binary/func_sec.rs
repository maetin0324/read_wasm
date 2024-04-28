use nom::IResult;
use nom_leb128::leb128_u32;


#[derive(Debug, PartialEq)]
pub struct Func {
  pub type_idx: u32,
}

impl Func {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Func>> {
    let (mut input, func_count) = leb128_u32(input)?;
    let mut funcs: Vec<Func> = Vec::new();

    for _ in 0..func_count {
      let type_idx: u32;
      (input, type_idx) = leb128_u32(input)?;

      funcs.push(Func { type_idx });
    }

    Ok((input, funcs))
  }
}