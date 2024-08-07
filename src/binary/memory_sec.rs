use core::panic;

use nom::{bytes::complete::take, IResult};
use nom_leb128::leb128_u32;


#[derive(Debug, Clone, PartialEq)]
pub struct MemorySec {
  pub min: u32,
  pub max: Option<u32>,
}

impl MemorySec {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<MemorySec>> {
    let (mut input, memory_count) = leb128_u32(input)?;
    let mut memories = Vec::new();

    for _ in 0..memory_count {
      let flags: &[u8];
      let min: u32;
      let max: u32;

      (input, flags) = take(1usize)(input)?;
      match flags[0] {
        0x00 => {
          (input, min) = leb128_u32(input)?;
          memories.push(MemorySec{min, max: None});
        }
        0x01 => {
          (input, min) = leb128_u32(input)?;
          (input, max) = leb128_u32(input)?;
          memories.push(MemorySec{min, max: Some(max)})
        }
        _ => panic!("unknown limits flags")
      }
    }
    Ok((input, memories))
  }
}