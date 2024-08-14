use nom::{bytes::complete::take, IResult};
use nom_leb128::leb128_u32;

#[derive(Debug, Clone, PartialEq)]
pub struct TableSec {
  pub min: u32,
  pub max: Option<u32>,
  pub reftype: RefType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefType {
  FuncRef,
}

impl TableSec {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<TableSec>> {
    let mut tables = Vec::new();
    let (mut input, count) = leb128_u32(input)?;
    for _ in 0..count {
      let (rest, reftype) = RefType::parse(input)?;
      let (rest, flag) = take(1usize)(rest)?;
      match flag[0] {
        0x70 => {
          let (rest, min) = leb128_u32(rest)?;
          tables.push(TableSec{min, max: None, reftype});
          input = rest;
        }
        0x01 => {
          let (rest, min) = leb128_u32(rest)?;
          let (rest, max) = leb128_u32(rest)?;
          tables.push(TableSec{min, max: Some(max), reftype});
          input = rest;
        }
        _ => panic!("unknown limits flags")
      }
      
    }

    Ok((input, tables))
  }
}

impl RefType {
  pub fn parse(input: &[u8]) -> IResult<&[u8], RefType> {
    let (input, reftype) = take(1usize)(input)?;
    match reftype[0] {
      0x70 => Ok((input, RefType::FuncRef)),
      _ => panic!("unknown reference type"),
    }
  }
}