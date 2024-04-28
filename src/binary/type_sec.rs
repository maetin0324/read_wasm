use nom::{
  bytes::complete::{tag, take, take_till},
  IResult
};
use nom_leb128::leb128_u32;

use super::value_type::ValueType;

#[derive(Debug, PartialEq, Clone)]
pub struct FuncType {
  pub param_types: Vec<ValueType>,
  pub return_types: Vec<ValueType>,
}

impl FuncType {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<FuncType>> {
    let (mut input,func_type_count) = leb128_u32(input)?;
    let mut func_types: Vec<FuncType> = Vec::new();
    for _ in 0..func_type_count {
      let param_count: u32;
      let params: &[u8];
      let return_count: u32;
      let returns: &[u8];

      (input, _) = take_till(|c| c == 0x60)(input)?;
      (input, _) = tag([0x60])(input)?;
      (input, param_count) = leb128_u32(input)?;
      (input, params) = take(param_count)(input)?;
      (input, return_count) = leb128_u32(input)?;
      (input, returns) = take(return_count)(input)?;

      let param_types = params.iter().map(|&x| ValueType::parse(x)).collect();
      let return_types = returns.iter().map(|&x| ValueType::parse(x)).collect();
      
      func_types.push(FuncType { param_types, return_types });
    }
    Ok((input, func_types))
  }
}