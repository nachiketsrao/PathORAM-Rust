mod block;
mod bucket;
mod crypto;
mod oram;
mod redis;

use indicatif::ProgressBar;
use oram::ORAM;
use std::error::Error;
use std::time::Instant;

const LOG_CAPACITY: i32 = 15; // Logarithm base 2 of capacity (1024 buckets)
const Z: i32 = 5;           // Number of blocks per bucket
const STASH_SIZE: i32 = 20;  // Maximum number of blocks in stash

fn main() -> Result<(), Box<dyn Error>> {
    // Note: Redis now expects a URL like "redis://localhost:6379"
    let mut oram = ORAM::new(LOG_CAPACITY, Z, STASH_SIZE, "redis://localhost:6379")?;
    let total_operations: u64 = 10000;

    // Measure PUT throughput.
    let write_bar = ProgressBar::new(total_operations);
    write_bar.set_message("Writing: ");
    let put_start = Instant::now();
    for i in 0..total_operations {
        let key = i as i32;
        let value = format!("Value{}", i);
        oram.put(key, value);
        write_bar.inc(1);
    }
    write_bar.finish_with_message("Write complete");
    let put_duration = put_start.elapsed();
    let put_throughput = total_operations as f64 / put_duration.as_secs_f64();
    println!("PUT throughput: {:.2} iterations/second", put_throughput);

    // Measure GET throughput.
    let read_bar = ProgressBar::new(total_operations);
    read_bar.set_message("Reading: ");
    let get_start = Instant::now();
    for i in 0..total_operations {
        let expected_value = format!("Value{}", i);
        let retrieved_value = oram.get(i as i32);
        if retrieved_value != expected_value {
            eprintln!(
                "Mismatched value for key {}: expected {}, got {}",
                i, expected_value, retrieved_value
            );
        }
        read_bar.inc(1);
    }
    read_bar.finish_with_message("Read complete");
    let get_duration = get_start.elapsed();
    let get_throughput = total_operations as f64 / get_duration.as_secs_f64();
    println!("GET throughput: {:.2} iterations/second", get_throughput);

    Ok(())
}
