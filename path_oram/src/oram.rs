use std::collections::{HashMap, HashSet};
use crate::block::Block;
use crate::bucket::Bucket;
use crate::crypto;
use crate::redis::RedisClient;

pub struct ORAM {
    pub log_capacity: i32,             // Height of the tree (log base 2 of capacity)
    pub z: i32,                        // Number of blocks per bucket
    pub redis_client: RedisClient,
    pub stash_map: HashMap<i32, Block>,
    pub stash_size: i32,               // Maximum number of blocks in the stash
    pub key_map: HashMap<i32, i32>,      // Mapping from block IDs to leaves
    pub block_ids: i32,                // Keeps track of the block IDs used so far
}

impl ORAM {
    pub fn new(log_capacity: i32, z: i32, stash_size: i32, redis_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let key = crypto::generate_random_key()?;
        let client = RedisClient::new(redis_addr, &key)?;
        let mut oram = ORAM {
            log_capacity,
            z,
            stash_size,
            redis_client: client,
            stash_map: HashMap::new(),
            key_map: HashMap::new(),
            block_ids: 0,
        };
        oram.initialize();
        Ok(oram)
    }

    pub fn initialize(&mut self) {
        let total_buckets = (1 << (self.log_capacity + 1)) - 1;
        for i in 0..total_buckets {
            let dummy_block = Block {
                block_id: -1,
                key: -1,
                value: String::new(),
            };
            let bucket = Bucket {
                blocks: vec![dummy_block; self.z as usize],
                real_block_count: 0,
            };
            let _ = self.redis_client.write_bucket_to_db(i, &bucket);
        }
    }

    pub fn get_depth(&self, bucket_index: i32) -> i32 {
        let mut depth = 0;
        while (1 << depth) - 1 <= bucket_index {
            depth += 1;
        }
        depth - 1
    }

    pub fn bucket_for_level_leaf(&self, level: i32, leaf: i32) -> i32 {
        ((leaf + (1 << self.log_capacity)) >> (self.log_capacity - level)) - 1
    }

    pub fn can_include(&self, entry_leaf: i32, leaf: i32, level: i32) -> bool {
        (entry_leaf >> (self.log_capacity - level)) == (leaf >> (self.log_capacity - level))
    }

    pub fn put(&mut self, key: i32, value: String) -> String {
        let new_block = Block {
            block_id: key,
            key,
            value,
        };
        let return_block = self.access(false, new_block.block_id, new_block);
        return_block.value
    }

    pub fn get(&mut self, key: i32) -> String {
        let dummy_block = Block {
            block_id: -1,
            key: -1,
            value: String::new(),
        };
        let return_block = self.access(true, key, dummy_block);
        return_block.value
    }

    pub fn read_path(&mut self, leaf: i32, put_in_stash: bool) -> Option<HashSet<i32>> {
        let max_leaf = (1 << self.log_capacity) - 1;
        if leaf < 0 || leaf > max_leaf {
            println!("invalid leaf value: {}, valid range is [0, {}]", leaf, max_leaf);
            return None;
        }
        let mut path = HashSet::new();
        for level in 0..=self.log_capacity {
            let bucket_index = self.bucket_for_level_leaf(level, leaf);
            path.insert(bucket_index);
        }
        if put_in_stash {
            for bucket_index in path.iter() {
                if let Ok(bucket_data) = self.redis_client.read_bucket_from_db(*bucket_index) {
                    for block in bucket_data.blocks.iter() {
                        if block.key != -1 {
                            self.stash_map.insert(block.key, block.clone());
                        }
                    }
                }
            }
        }
        Some(path)
    }

    pub fn write_path(&mut self, leaf: i32) {
        let mut current_stash = self.stash_map.clone();
        let mut to_delete = HashSet::new();
        let mut requests: Vec<(i32, Bucket)> = Vec::new();

        for level in (0..=self.log_capacity).rev() {
            let mut to_insert: HashMap<i32, Block> = HashMap::new();
            let mut to_delete_local: Vec<i32> = Vec::new();

            for (&key, current_block) in current_stash.iter() {
                let current_block_leaf = *self.key_map.get(&current_block.key).unwrap_or(&0);
                if self.can_include(current_block_leaf, leaf, level) {
                    to_insert.insert(current_block.key, current_block.clone());
                    to_delete.insert(current_block.key);
                    to_delete_local.push(key);
                    if to_insert.len() == self.z as usize {
                        break;
                    }
                }
            }
            for key in to_delete_local {
                current_stash.remove(&key);
            }
            let bucket_id = self.bucket_for_level_leaf(level, leaf);
            let mut bucket = Bucket {
                blocks: Vec::with_capacity(self.z as usize),
                real_block_count: 0,
            };
            for block in to_insert.values().take(self.z as usize) {
                bucket.blocks.push(block.clone());
            }
            while bucket.blocks.len() < self.z as usize {
                bucket.blocks.push(Block {
                    block_id: -1,
                    key: -1,
                    value: String::new(),
                });
            }
            requests.push((bucket_id, bucket));
        }
        for (bucket_id, bucket) in requests {
            if let Err(e) = self.redis_client.write_bucket_to_db(bucket_id, &bucket) {
                println!("Error writing bucket {}: {:?}", bucket_id, e);
            }
        }
        for key in to_delete {
            self.stash_map.remove(&key);
        }
    }

    pub fn access(&mut self, read: bool, block_id: i32, data: Block) -> Block {
        let previous_position_leaf = if let Some(&leaf) = self.key_map.get(&block_id) {
            leaf
        } else {
            let new_leaf = crypto::get_random_int(1 << (self.log_capacity - 1));
            self.stash_map.insert(block_id, data.clone());
            new_leaf
        };
        self.key_map.insert(block_id, crypto::get_random_int(1 << (self.log_capacity - 1)));
        let _ = self.read_path(previous_position_leaf, true);
        if !read {
            self.stash_map.insert(block_id, data.clone());
        }
        let return_value = self.stash_map.get(&block_id).cloned().unwrap();
        self.write_path(previous_position_leaf);
        return_value
    }
}
