#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub block_id: i32,
    pub key: i32, // dummy can have key -1
    pub value: String,
}
