use serde::{Deserialize, Serialize};
use crate::binary::instructions::{Block, BlockType};

use super::value::Value;

#[derive(Debug, Clone, PartialEq , Serialize, Deserialize)]
pub struct BlockFrame {
  pub value_stack_evac: Vec<Value>,
  pub return_type: BlockType,
}

impl BlockFrame {
  pub fn new(value_stack_evac: Vec<Value>, block: Block) -> BlockFrame {
    BlockFrame {
      value_stack_evac,
      return_type: block.block_type,
    }
  }
}