use crate::block::Block;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bucket {
    pub blocks: Vec<Block>,
    pub real_block_count: i32,
}
