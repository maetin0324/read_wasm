use nom::IResult;
use nom::bytes::complete::take;
use nom_leb128::leb128_u32;


#[derive(Debug, PartialEq)]
pub struct Import {
  pub module: String,
  pub field: String,
  pub desc: ImportDesc,
}

#[derive(Debug, PartialEq)]
pub enum ImportDesc {
  Func(u32),
  Table,
  Memory,
  Global,
}

impl Import {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Import>> {
    let (mut input, import_count) = leb128_u32(input)?;
    let mut imports: Vec<Import> = Vec::new();

    for _ in 0..import_count {
      let module_len: u32;
      let module: &[u8];
      let field_len: u32;
      let field: &[u8];
      let kind: &[u8];
      let type_idx: u32;

      (input, module_len) = leb128_u32(input)?;
      (input, module) = take(module_len as usize)(input)?;
      (input, field_len) = leb128_u32(input)?;
      (input, field) = take(field_len as usize)(input)?;
      (input, kind) = take(1_usize)(input)?;

      let module = String::from_utf8(module.to_vec()).unwrap();
      let field = String::from_utf8(field.to_vec()).unwrap();
      let kind = kind[0];

      match kind {
        0x00 => {
          (input, type_idx) = leb128_u32(input)?;
          imports.push(Import { module, field, desc: ImportDesc::Func(type_idx) });
        },
        0x01 => {
          imports.push(Import { module, field, desc: ImportDesc::Table });
        },
        0x02 => {
          imports.push(Import { module, field, desc: ImportDesc::Memory });
        },
        0x03 => {
          imports.push(Import { module, field, desc: ImportDesc::Global });
        },
        _ => panic!("Invalid import kind: {:#x?}", kind),
      }
    }

    Ok((input, imports))
  }
}

