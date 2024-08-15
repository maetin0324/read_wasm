use core::panic;
use serde::{Deserialize, Serialize};

use nom::{
  bytes::complete::tag, number::complete::le_u8, IResult
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
  pub jump_pc: usize,
  pub is_loop: bool,
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
  BrTable(Vec<u32>, u32),
  Return,
  Call(u32),
  CallIndirect(u32, u32),
  Drop,
  Select,
  SelectValtype(Vec<ValueType>),
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
  MemoryInit(u32),
  DataDrop(u32),
  MemoryCopy,
  MemoryFill,
  I32Const(i32),
  I64Const(i64),
  F32Const(f32),
  F64Const(f64),
  I32Eqz,
  I32Eq,
  I32Ne,
  I32LtS,
  I32LtU,
  I32GtS,
  I32GtU,
  I32LeS,
  I32LeU,
  I32GeS,
  I32GeU,
  I64Eqz,
  I64Eq,
  I64Ne,
  I64LtS,
  I64LtU,
  I64GtS,
  I64GtU,
  I64LeS,
  I64LeU,
  I64GeS,
  I64GeU,
  F32Eq,
  F32Ne,
  F32Lt,
  F32Gt,
  F32Le,
  F32Ge,
  F64Eq,
  F64Ne,
  F64Lt,
  F64Gt,
  F64Le,
  F64Ge,
  I32Clz,
  I32Ctz,
  I32Popcnt,
  I32Add,
  I32Sub,
  I32Mul,
  I32DivS,
  I32DivU,
  I32RemS,
  I32RemU,
  I32And,
  I32Or,
  I32Xor,
  I32Shl,
  I32ShrS,
  I32ShrU,
  I32Rotl,
  I32Rotr,
  I64Clz,
  I64Ctz,
  I64Popcnt,
  I64Add,
  I64Sub,
  I64Mul,
  I64DivS,
  I64DivU,
  I64RemS,
  I64RemU,
  I64And,
  I64Or,
  I64Xor,
  I64Shl,
  I64ShrS,
  I64ShrU,
  I64Rotl,
  I64Rotr,
  F32Abs,
  F32Neg,
  F32Ceil,
  F32Floor,
  F32Trunc,
  F32Nearest,
  F32Sqrt,
  F32Add,
  F32Sub,
  F32Mul,
  F32Div,
  F32Min,
  F32Max,
  F32Copysign,
  F64Abs,
  F64Neg,
  F64Ceil,
  F64Floor,
  F64Trunc,
  F64Nearest,
  F64Sqrt,
  F64Add,
  F64Sub,
  F64Mul,
  F64Div,
  F64Min,
  F64Max,
  F64Copysign,
  I32WrapI64,
  I32TruncF32S,
  I32TruncF32U,
  I32TruncF64S,
  I32TruncF64U,
  I64ExtendI32S,
  I64ExtendI32U,
  I64TruncF32S,
  I64TruncF32U,
  I64TruncF64S,
  I64TruncF64U,
  F32ConvertI32S,
  F32ConvertI32U,
  F32ConvertI64S,
  F32ConvertI64U,
  F32DemoteF64,
  F64ConvertI32S,
  F64ConvertI32U,
  F64ConvertI64S,
  F64ConvertI64U,
  F64PromoteF32,
  I32ReinterpretF32,
  I64ReinterpretF64,
  F32ReinterpretI32,
  F64ReinterpretI64,
  I32Extend8S,
  I32Extend16S,
  I64Extend8S,
  I64Extend16S,
  I64Extend32S,
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
    let mut label_stack: Vec<Option<usize>> = Vec::new();
    loop {
      let (i, instr) = Instructions::parse_single(input)?;
      let instr = match instr {
        Instructions::Block(_) => {
          label_stack.push(Some(instructions.len()));
          instr
        },
        Instructions::Loop(lblock) => {
          label_stack.push(None);
          let nlb = Block { jump_pc: instructions.len() + 1, ..lblock };
          Instructions::Loop(nlb)
        },
        Instructions::End => {
          match label_stack.pop() {
            Some(Some(pc)) => match &instructions[pc] {
              Instructions::Block(block) => {
                let block = Block { jump_pc: instructions.len(), ..block.clone() };
                instructions[pc] = Instructions::Block(block);
              },
              _ => panic!("Invalid block type"),
            },
            Some(None) => {},
            None => {},
            }
          Instructions::End
        },
        _ => instr,
      };
      instructions.push(instr);
      input = i;
      if input.is_empty() {
        break;
      }
    }
    Ok((input, instructions))
  }

  pub fn parse_init(input: &[u8]) -> IResult<&[u8], Vec<Instructions>> {
    let mut instrs = Vec::new();
    let mut input = input;
    loop {
      let (i, instr) = Instructions::parse_single(input)?;
      input = i;
      if &instr == &Instructions::End {
        instrs.push(instr);
        break;
      }
      instrs.push(instr);
    }
    Ok((input, instrs))
  }

  fn parse_single(input: &[u8]) -> IResult<&[u8], Instructions> {
    let (input, opcode) = le_u8(input)?;

    match opcode {
      0x00 => Ok((input, Instructions::Unreachable)),
      0x01 => Ok((input, Instructions::Nop)),
      0x02 => {
        let (input, block) = Block::parse(input, false)?;
        Ok((input, Instructions::Block(block)))
      },
      0x03 => {
        let (input, block) = Block::parse(input, true)?;
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
      0x0e => {
        let mut labels = Vec::new();
        let (mut input, count) = leb128_u32(input)?;
        for _ in 0..count {
          let (i, label_idx) = leb128_u32(input)?;
          labels.push(label_idx);
          input = i;
        }
        let (input, default) = leb128_u32(input)?;
        Ok((input, Instructions::BrTable(labels, default)))
      }
      0x0f => Ok((input, Instructions::Return)),
      0x10 => {
        let (input, func_idx) = leb128_u32(input)?;
        Ok((input, Instructions::Call(func_idx)))
      }
      0x11 => {
        let (input, type_idx) = leb128_u32(input)?;
        let (input, table_idx) = leb128_u32(input)?;
        Ok((input, Instructions::CallIndirect(type_idx, table_idx)))
      }
      0x1a => Ok((input, Instructions::Drop)),
      0x1b => Ok((input, Instructions::Select)),
      0x1c => {
        let (input, count) = leb128_u32(input)?;
        let (input, valtypes) = ValueType::parse_vec(input, count)?;
        Ok((input, Instructions::SelectValtype(valtypes)))
      },
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
      0x3f => {
        let (input, _) = tag([0x00])(input)?;
        Ok((input, Instructions::MemorySize))
      },
      0x40 => {
        let (input, _) = tag([0x00])(input)?;
        Ok((input, Instructions::MemoryGrow))
      },
      0x41 => {
        let (input, val) = leb128_i32(input)?;
        Ok((input, Instructions::I32Const(val)))
      },
      0x42 => {
        let (input, val) = leb128_i64(input)?;
        Ok((input, Instructions::I64Const(val)))
      },
      0x43 => {
        let (input, val) = leb128_i32(input)?;
        Ok((input, Instructions::F32Const(val as f32)))
      },
      0x44 => {
        let (input, val) = leb128_i64(input)?;
        Ok((input, Instructions::F64Const(val as f64)))
      },
      0x45 => Ok((input, Instructions::I32Eqz)),
      0x46 => Ok((input, Instructions::I32Eq)),
      0x47 => Ok((input, Instructions::I32Ne)),
      0x48 => Ok((input, Instructions::I32LtS)),
      0x49 => Ok((input, Instructions::I32LtU)),
      0x4a => Ok((input, Instructions::I32GtS)),
      0x4b => Ok((input, Instructions::I32GtU)),
      0x4c => Ok((input, Instructions::I32LeS)),
      0x4d => Ok((input, Instructions::I32LeU)),
      0x4e => Ok((input, Instructions::I32GeS)),
      0x4f => Ok((input, Instructions::I32GeU)),
      0x50 => Ok((input, Instructions::I64Eqz)),
      0x51 => Ok((input, Instructions::I64Eq)),
      0x52 => Ok((input, Instructions::I64Ne)),
      0x53 => Ok((input, Instructions::I64LtS)),
      0x54 => Ok((input, Instructions::I64LtU)),
      0x55 => Ok((input, Instructions::I64GtS)),
      0x56 => Ok((input, Instructions::I64GtU)),
      0x57 => Ok((input, Instructions::I64LeS)),
      0x58 => Ok((input, Instructions::I64LeU)),
      0x59 => Ok((input, Instructions::I64GeS)),
      0x5a => Ok((input, Instructions::I64GeU)),
      0x5b => Ok((input, Instructions::F32Eq)),
      0x5c => Ok((input, Instructions::F32Ne)),
      0x5d => Ok((input, Instructions::F32Lt)),
      0x5e => Ok((input, Instructions::F32Gt)),
      0x5f => Ok((input, Instructions::F32Le)),
      0x60 => Ok((input, Instructions::F32Ge)),
      0x61 => Ok((input, Instructions::F64Eq)),
      0x62 => Ok((input, Instructions::F64Ne)),
      0x63 => Ok((input, Instructions::F64Lt)),
      0x64 => Ok((input, Instructions::F64Gt)),
      0x65 => Ok((input, Instructions::F64Le)),
      0x66 => Ok((input, Instructions::F64Ge)),
      0x67 => Ok((input, Instructions::I32Clz)),
      0x68 => Ok((input, Instructions::I32Ctz)),
      0x69 => Ok((input, Instructions::I32Popcnt)),
      0x6a => Ok((input, Instructions::I32Add)),
      0x6b => Ok((input, Instructions::I32Sub)),
      0x6c => Ok((input, Instructions::I32Mul)),
      0x6d => Ok((input, Instructions::I32DivS)),
      0x6e => Ok((input, Instructions::I32DivU)),
      0x6f => Ok((input, Instructions::I32RemS)),
      0x70 => Ok((input, Instructions::I32RemU)),
      0x71 => Ok((input, Instructions::I32And)),
      0x72 => Ok((input, Instructions::I32Or)),
      0x73 => Ok((input, Instructions::I32Xor)),
      0x74 => Ok((input, Instructions::I32Shl)),
      0x75 => Ok((input, Instructions::I32ShrS)),
      0x76 => Ok((input, Instructions::I32ShrU)),
      0x77 => Ok((input, Instructions::I32Rotl)),
      0x78 => Ok((input, Instructions::I32Rotr)),
      0x79 => Ok((input, Instructions::I64Clz)),
      0x7a => Ok((input, Instructions::I64Ctz)),
      0x7b => Ok((input, Instructions::I64Popcnt)),
      0x7c => Ok((input, Instructions::I64Add)),
      0x7d => Ok((input, Instructions::I64Sub)),
      0x7e => Ok((input, Instructions::I64Mul)),
      0x7f => Ok((input, Instructions::I64DivS)),
      0x80 => Ok((input, Instructions::I64DivU)),
      0x81 => Ok((input, Instructions::I64RemS)),
      0x82 => Ok((input, Instructions::I64RemU)),
      0x83 => Ok((input, Instructions::I64And)),
      0x84 => Ok((input, Instructions::I64Or)),
      0x85 => Ok((input, Instructions::I64Xor)),
      0x86 => Ok((input, Instructions::I64Shl)),
      0x87 => Ok((input, Instructions::I64ShrS)),
      0x88 => Ok((input, Instructions::I64ShrU)),
      0x89 => Ok((input, Instructions::I64Rotl)),
      0x8a => Ok((input, Instructions::I64Rotr)),
      0x8b => Ok((input, Instructions::F32Abs)),
      0x8c => Ok((input, Instructions::F32Neg)),
      0x8d => Ok((input, Instructions::F32Ceil)),
      0x8e => Ok((input, Instructions::F32Floor)),
      0x8f => Ok((input, Instructions::F32Trunc)),
      0x90 => Ok((input, Instructions::F32Nearest)),
      0x91 => Ok((input, Instructions::F32Sqrt)),
      0x92 => Ok((input, Instructions::F32Add)),
      0x93 => Ok((input, Instructions::F32Sub)),
      0x94 => Ok((input, Instructions::F32Mul)),
      0x95 => Ok((input, Instructions::F32Div)),
      0x96 => Ok((input, Instructions::F32Min)),
      0x97 => Ok((input, Instructions::F32Max)),
      0x98 => Ok((input, Instructions::F32Copysign)),
      0x99 => Ok((input, Instructions::F64Abs)),
      0x9a => Ok((input, Instructions::F64Neg)),
      0x9b => Ok((input, Instructions::F64Ceil)),
      0x9c => Ok((input, Instructions::F64Floor)),
      0x9d => Ok((input, Instructions::F64Trunc)),
      0x9e => Ok((input, Instructions::F64Nearest)),
      0x9f => Ok((input, Instructions::F64Sqrt)),
      0xa0 => Ok((input, Instructions::F64Add)),
      0xa1 => Ok((input, Instructions::F64Sub)),
      0xa2 => Ok((input, Instructions::F64Mul)),
      0xa3 => Ok((input, Instructions::F64Div)),
      0xa4 => Ok((input, Instructions::F64Min)),
      0xa5 => Ok((input, Instructions::F64Max)),
      0xa6 => Ok((input, Instructions::F64Copysign)),
      0xa7 => Ok((input, Instructions::I32WrapI64)),
      0xa8 => Ok((input, Instructions::I32TruncF32S)),
      0xa9 => Ok((input, Instructions::I32TruncF32U)),
      0xaa => Ok((input, Instructions::I32TruncF64S)),
      0xab => Ok((input, Instructions::I32TruncF64U)),
      0xac => Ok((input, Instructions::I64ExtendI32S)),
      0xad => Ok((input, Instructions::I64ExtendI32U)),
      0xae => Ok((input, Instructions::I64TruncF32S)),
      0xaf => Ok((input, Instructions::I64TruncF32U)),
      0xb0 => Ok((input, Instructions::I64TruncF64S)),
      0xb1 => Ok((input, Instructions::I64TruncF64U)),
      0xb2 => Ok((input, Instructions::F32ConvertI32S)),
      0xb3 => Ok((input, Instructions::F32ConvertI32U)),
      0xb4 => Ok((input, Instructions::F32ConvertI64S)),
      0xb5 => Ok((input, Instructions::F32ConvertI64U)),
      0xb6 => Ok((input, Instructions::F32DemoteF64)),
      0xb7 => Ok((input, Instructions::F64ConvertI32S)),
      0xb8 => Ok((input, Instructions::F64ConvertI32U)),
      0xb9 => Ok((input, Instructions::F64ConvertI64S)),
      0xba => Ok((input, Instructions::F64ConvertI64U)),
      0xbb => Ok((input, Instructions::F64PromoteF32)),
      0xbc => Ok((input, Instructions::I32ReinterpretF32)),
      0xbd => Ok((input, Instructions::I64ReinterpretF64)),
      0xbe => Ok((input, Instructions::F32ReinterpretI32)),
      0xbf => Ok((input, Instructions::F64ReinterpretI64)),
      0xc0 => Ok((input, Instructions::I32Extend8S)),
      0xc1 => Ok((input, Instructions::I32Extend16S)),
      0xc2 => Ok((input, Instructions::I64Extend8S)),
      0xc3 => Ok((input, Instructions::I64Extend16S)),
      0xc4 => Ok((input, Instructions::I64Extend32S)),

      0xfc => {
        let (input, byte) = leb128_u32(input)?;
        match byte {
          0x08 => {
            let (input, dataidx) = leb128_u32(input)?;
            let (input, _) = tag([0x00])(input)?;
            Ok((input, Instructions::MemoryInit(dataidx)))
          }
          0x09 => {
            let (input, dataidx) = leb128_u32(input)?;
            let (input, _) = tag([0x00])(input)?;
            Ok((input, Instructions::DataDrop(dataidx)))
          }
          0x0a => {
            let (input, _) = tag([0x00, 0x00])(input)?;
            Ok((input, Instructions::MemoryCopy))
          },
          0x0b => {
            let (input, _) = tag([0x00])(input)?;
            Ok((input, Instructions::MemoryFill))
          },
          _ => panic!("Unknown opcode: 0xfc {:#x?}", byte),
        }
      }
      
      _ => {
        let next = input.iter().take(10).map(|x| format!("{:#x?}", x)).collect::<Vec<String>>().join(", ");
        panic!("Unknown opcode: {:#x?}, next: {:#?}", opcode, next);
      }
    }
  }
}

impl Block {
  pub fn parse(input: &[u8], is_loop: bool) -> IResult<&[u8], Block> {
    let (input, block_type) = le_u8(input)?;
    let block_type = match block_type {
      0x40 => BlockType::Void,
      _ => BlockType::Value(ValueType::parse(block_type)),
    };
    Ok((input, Block { block_type , jump_pc: 0, is_loop }))
  }
}
