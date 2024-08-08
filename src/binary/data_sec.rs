use nom::{bytes::complete::{tag, take}, IResult};
use nom_leb128::{leb128_u32, leb128_u64};

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
  pub memory_index: u32,
  pub offset: u32,
  pub init: Vec<u8>,
}

impl Data {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Data>> {
    let (mut input, count) = leb128_u32(input)?;
    let mut data = vec![];
    for _ in 0..count {
        let (rest, memory_index) = leb128_u32(input)?;
        let (rest, offset) = decode_expr(rest)?;
        let (rest, size) = leb128_u32(rest)?;
        let (rest, init) = take(size)(rest)?;
        data.push(Data {
            memory_index,
            offset,
            init: init.into(),
        });
        input = rest;
    }
    Ok((input, data))
  }
}

fn decode_expr(input: &[u8]) -> IResult<&[u8], u32> {
  let (mut input, expr) = take(1usize)(input)?;
  let offset: u32;
  match expr[0] {
    0x41 => {(input, offset) = leb128_u32(input)?;},
    0x42 => { 
      let tmp: u64;
      (input, tmp) = leb128_u64(input)?;
      offset = tmp as u32;
    }
    _ => panic!("unknown expression")
  }
  let (input, _) = tag([0x0b])(input)?;
  Ok((input, offset))
}