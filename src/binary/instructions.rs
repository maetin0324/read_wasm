use core::panic;

use nom::{
  number::complete::le_u8, IResult,
};

use nom_leb128::{leb128_i32, leb128_i64};

#[derive(Debug, PartialEq)]
pub enum Instructions {
  I32Const(i32),
  I64Const(i64),
  I32Add,
  I64Add,
}

impl Instructions {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Instructions>> {
    let mut instructions: Vec<Instructions> = Vec::new();
    let mut input = input;
    loop {
      let (i, instr) = Instructions::parse_single(input)?;
      instructions.push(instr);
      input = i;
      if input.len() == 0 {
        break;
      }
    }
    Ok((input, instructions))
  }

  fn parse_single(input: &[u8]) -> IResult<&[u8], Instructions> {
    let (input, opcode) = le_u8(input)?;

    match opcode {
      0x41 => {
        let (input, val) = leb128_i32(input)?;
        Ok((input, Instructions::I32Const(val)))
      },
      0x42 => {
        let (input, val) = leb128_i64(input)?;
        Ok((input, Instructions::I64Const(val)))
      },
      0x6a => Ok((input, Instructions::I32Add)),
      0x7c => Ok((input, Instructions::I64Add)),
      _ => {
        panic!("Unknown opcode: {:#x?}", opcode);
      }
    }
  }
}

