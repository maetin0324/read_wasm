use nom::{
  bytes::complete::take, number::complete::le_u8, IResult
};
use nom_leb128::leb128_u32;

#[derive(Debug, PartialEq)]
pub struct ExportFunc{
  pub name: String,
  pub desc: ExportDesc,
  pub func_idx: u32,
}

#[derive(Debug, PartialEq)]
pub enum ExportDesc {
  Func,
  Table,
  Mem,
  Global,
}

impl ExportFunc {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<ExportFunc>> {
    let (mut input, func_count) = leb128_u32(input)?;
    let mut funcs: Vec<ExportFunc> = Vec::new();

    for _ in 0..func_count {
      let string_len: u32;
      let name_buf: &[u8];
      let desc: ExportDesc;
      let func_idx: u32;

      (input, string_len) = leb128_u32(input)?;
      (input, name_buf) = take(string_len)(input)?;

      let name = String::from_utf8(name_buf.to_vec()).unwrap();

      (input, desc) = ExportDesc::parse(input)?;
      (input, func_idx) = leb128_u32(input)?;

      funcs.push(ExportFunc { name, desc, func_idx });
    }

    Ok((input, funcs))
  }
}

impl ExportDesc {
  pub fn parse(input: &[u8]) -> IResult<&[u8], ExportDesc> {
    let (input, desc) = le_u8(input)?;

    match desc {
      0x00 => Ok((input, ExportDesc::Func)),
      0x01 => Ok((input, ExportDesc::Table)),
      0x02 => Ok((input, ExportDesc::Mem)),
      0x03 => Ok((input, ExportDesc::Global)),
      _ => panic!("Unknown export description: {:#x?}", desc),
    }
  }
}