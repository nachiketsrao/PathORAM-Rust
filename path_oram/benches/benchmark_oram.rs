use criterion::{black_box, criterion_group, criterion_main, Criterion};
use path_oram::oram::ORAM;
use std::error::Error;

const LOG_CAPACITY: i32 = 10;
const Z: i32 = 5;
const STASH_SIZE: i32 = 20;

fn benchmark_put(c: &mut Criterion) -> Result<(), Box<dyn Error>> {
    let mut oram = ORAM::new(LOG_CAPACITY, Z, STASH_SIZE, "redis://localhost:6379")?;
    c.bench_function("oram put", |b| {
        b.iter(|| {
            for i in 0..1000 {
                // Use black_box to prevent compiler optimizations.
                oram.put(black_box(i), black_box(format!("Value{}", i)));
            }
        })
    });
    Ok(())
}

fn benchmark_get(c: &mut Criterion) -> Result<(), Box<dyn Error>> {
    let mut oram = ORAM::new(LOG_CAPACITY, Z, STASH_SIZE, "redis://localhost:6379")?;
    // Prepopulate the ORAM.
    for i in 0..1000 {
        oram.put(i, format!("Value{}", i));
    }
    c.bench_function("oram get", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let _ = oram.get(black_box(i));
            }
        })
    });
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let _ = benchmark_put(c);
    let _ = benchmark_get(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
