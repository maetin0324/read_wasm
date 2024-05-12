use core::panic;

use nom::{
  number::complete::le_u8, IResult,
};

use nom_leb128::{leb128_i32, leb128_i64, leb128_u32};

use super::value_type::ValueType;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
  Void,
  Value(ValueType)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  pub block_type: BlockType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instructions {
  Unreachable,
  Nop,
  Block(Block),
  Loop(Block),
  End,
  Br(u32),
  BrIf(u32),
  Return,
  Call(u32),
  Drop,
  I32Const(i32),
  I64Const(i64),
  I32Add,
  I64Add,
  LocalGet(u32),
}

impl Instructions {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Instructions>> {
    let mut instructions: Vec<Instructions> = Vec::new();
    let mut input = input;
    loop {
      let (i, instr) = Instructions::parse_single(input)?;
      instructions.push(instr);
      input = i;
      if input.is_empty() {
        break;
      }
    }
    Ok((input, instructions))
  }

  fn parse_single(input: &[u8]) -> IResult<&[u8], Instructions> {
    let (input, opcode) = le_u8(input)?;

    match opcode {
      0x00 => Ok((input, Instructions::Unreachable)),
      0x01 => Ok((input, Instructions::Nop)),
      0x02 => {
        let (input, block) = Block::parse(input)?;
        Ok((input, Instructions::Block(block)))
      },
      0x03 => {
        let (input, block) = Block::parse(input)?;
        Ok((input, Instructions::Loop(block)))
      },
      0x0b => Ok((input, Instructions::End)),
      0x0c => {
        let (input, label_idx) = leb128_u32(input)?;
        Ok((input, Instructions::Br(label_idx)))
      },
      0x0d => {
        let (input, label_idx) = leb128_u32(input)?;
        Ok((input, Instructions::BrIf(label_idx)))
      },
      0x0f => Ok((input, Instructions::Return)),
      0x10 => {
        let (input, func_idx) = leb128_u32(input)?;
        Ok((input, Instructions::Call(func_idx)))
      }
      0x1a => Ok((input, Instructions::Drop)),
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
      0x20 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::LocalGet(val)))
      },
      _ => {
        panic!("Unknown opcode: {:#x?}", opcode);
      }
    }
  }
}

impl Block {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Block> {
    let (input, block_type) = le_u8(input)?;
    let block_type = match block_type {
      0x40 => BlockType::Void,
      _ => BlockType::Value(ValueType::parse(block_type)),
    };
    Ok((input, Block { block_type }))
  }
}

