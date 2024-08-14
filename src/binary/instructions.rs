use core::panic;
use serde::{Deserialize, Serialize};

use nom::{
  number::complete::le_u8, IResult,
};

use nom_leb128::{leb128_i32, leb128_i64, leb128_u32};

use super::value_type::ValueType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
  Void,
  Value(ValueType),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
  pub block_type: BlockType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
  I32Load{align: u32, offset: u32},
  I64Load{align: u32, offset: u32},
  F32Load{align: u32, offset: u32},
  F64Load{align: u32, offset: u32},
  I32Load8S{align: u32, offset: u32},
  I32Load8U{align: u32, offset: u32},
  I32Load16S{align: u32, offset: u32},
  I32Load16U{align: u32, offset: u32},
  I64Load8S{align: u32, offset: u32},
  I64Load8U{align: u32, offset: u32},
  I64Load16S{align: u32, offset: u32},
  I64Load16U{align: u32, offset: u32},
  I64Load32S{align: u32, offset: u32},
  I64Load32U{align: u32, offset: u32},
  I32Store{align: u32, offset: u32},
  I64Store{align: u32, offset: u32},
  F32Store{align: u32, offset: u32},
  F64Store{align: u32, offset: u32},
  I32Store8{align: u32, offset: u32},
  I32Store16{align: u32, offset: u32},
  I64Store8{align: u32, offset: u32},
  I64Store16{align: u32, offset: u32},
  I64Store32{align: u32, offset: u32},
  MemorySize,
  MemoryGrow,
  I32Const(i32),
  I32Eqz,
  I32Eq,
  I64Eqz,
  I64Eq,
  I64Const(i64),
  I32Add,
  I32Sub,
  I64Add,
  I64Sub,
  LocalGet(u32),
  LocalSet(u32),
  LocalTee(u32),
  GlobalGet(u32),
  GlobalSet(u32),
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
      0x20 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::LocalGet(val)))
      },
      0x21 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::LocalSet(val)))
      },
      0x22 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::LocalTee(val)))
      },
      0x23 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::GlobalGet(val)))
      },
      0x24 => {
        let (input, val) = leb128_u32(input)?;
        Ok((input, Instructions::GlobalSet(val)))
      },
      0x28 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Load { align, offset }))
      },
      0x29 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load { align, offset }))
      },
      0x2a => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::F32Load { align, offset }))
      },
      0x2b => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::F64Load { align, offset }))
      },
      0x2c => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Load8S { align, offset }))
      },
      0x2d => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Load8U { align, offset }))
      },
      0x2e => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Load16S { align, offset }))
      },
      0x2f => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Load16U { align, offset }))
      },
      0x30 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load8S { align, offset }))
      },
      0x31 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load8U { align, offset }))
      },
      0x32 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load16S { align, offset }))
      },
      0x33 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load16U { align, offset }))
      },
      0x34 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load32S { align, offset }))
      },
      0x35 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Load32U { align, offset }))
      },
      0x36 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Store { align, offset }))
      },
      0x37 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Store { align, offset }))
      },
      0x38 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::F32Store { align, offset }))
      },
      0x39 => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::F64Store { align, offset }))
      },
      0x3a => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Store8 { align, offset }))
      },
      0x3b => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I32Store16 { align, offset }))
      },
      0x3c => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Store8 { align, offset }))
      },
      0x3d => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Store16 { align, offset }))
      },
      0x3e => {
        let (input, align) = leb128_u32(input)?;
        let (input, offset) = leb128_u32(input)?;
        Ok((input, Instructions::I64Store32 { align, offset }))
      },
      0x3f => Ok((input, Instructions::MemorySize)),
      0x40 => Ok((input, Instructions::MemoryGrow)),
      0x41 => {
        let (input, val) = leb128_i32(input)?;
        Ok((input, Instructions::I32Const(val)))
      },
      0x42 => {
        let (input, val) = leb128_i64(input)?;
        Ok((input, Instructions::I64Const(val)))
      },
      0x45 => Ok((input, Instructions::I32Eqz)),
      0x46 => Ok((input, Instructions::I32Eq)),
      0x50 => Ok((input, Instructions::I64Eqz)),
      0x51 => Ok((input, Instructions::I64Eq)),
      0x6a => Ok((input, Instructions::I32Add)),
      0x6b => Ok((input, Instructions::I32Sub)),
      0x7c => Ok((input, Instructions::I64Add)),
      0x7d => Ok((input, Instructions::I64Sub)),
      
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

