use redis::Commands;
use std::error::Error;
use crate::bucket::Bucket;
use crate::crypto;

pub struct RedisClient {
    client: redis::Client,
    conn: redis::Connection,
    encryption_key: Vec<u8>,
}

impl RedisClient {
    pub fn new(redis_addr: &str, encryption_key: &[u8]) -> Result<Self, Box<dyn Error>> {
        let client = redis::Client::open(redis_addr)?;
        let mut conn = client.get_connection()?;
        // Annotate query with <String> to avoid never type fallback warnings.
        let _: String = redis::cmd("PING").query::<String>(&mut conn)?;
        Ok(RedisClient {
            client,
            conn,
            encryption_key: encryption_key.to_vec(),
        })
    }

    pub fn write_bucket_to_db(&mut self, index: i32, bucket: &Bucket) -> Result<(), Box<dyn Error>> {
        let data = serde_json::to_vec(bucket)?;
        let encrypted_data = crypto::encrypt(&data, &self.encryption_key)?;
        let key = format!("bucket:{}", index);
        redis::cmd("SET")
            .arg(key)
            .arg(encrypted_data)
            .query::<()>(&mut self.conn)?;
        Ok(())
    }

    pub fn read_bucket_from_db(&mut self, index: i32) -> Result<Bucket, Box<dyn Error>> {
        let key = format!("bucket:{}", index);
        let encrypted_data: Vec<u8> = redis::cmd("GET").arg(&key).query(&mut self.conn)?;
        let decrypted_data = crypto::decrypt(&encrypted_data, &self.encryption_key)?;
        let bucket: Bucket = serde_json::from_slice(&decrypted_data)?;
        Ok(bucket)
    }

    pub fn close(mut self) -> Result<(), Box<dyn Error>> {
        redis::cmd("QUIT").query::<()>(&mut self.conn)?;
        Ok(())
    }
}
