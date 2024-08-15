use nom::{bytes::complete::take, IResult};
use nom_leb128::leb128_u32;

use super::{instructions::Instructions, value_type::ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVar {
    pub valtype: ValueType,
    pub mutability: bool,
    pub init: Vec<Instructions>
}

impl GlobalVar {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<GlobalVar>> {
    let (mut input, count) = leb128_u32(input)?;
    let mut globals: Vec<GlobalVar> = Vec::new();

    for _ in 0..count {
      let (rest, valtype) = take(1usize)(input)?;
      let valtype = ValueType::parse(valtype[0]);
      let (rest, mutability) = take(1usize)(rest)?;
      let mutability = mutability[0];
      let (rest, init) = Instructions::parse_init(rest).unwrap();

      globals.push(GlobalVar { valtype, mutability: mutability == 1, init });
      input = rest;
    }
    Ok((input, globals))
    
  }
}