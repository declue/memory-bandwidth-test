use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Instant;

use rand::{Rng, rngs::StdRng, SeedableRng};
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

fn main() {
    let num_elements = 100 * 1024 * 1024; // 100MB
    let data = Arc::new((0..num_elements).map(|_| AtomicU8::new(0)).collect::<Vec<_>>());

    // 순차 쓰기
    let start = Instant::now();
    data.par_iter().enumerate().for_each(|(i, x)| x.store((i % 256) as u8, Ordering::SeqCst));
    let seq_write_duration = start.elapsed();
    let seq_write_throughput = num_elements as f64 / seq_write_duration.as_secs_f64();

    // 순차 읽기
    let start = Instant::now();
    let sum_seq_read: u64 = data.par_iter().map(|x| x.load(Ordering::SeqCst) as u64).sum();
    let seq_read_duration = start.elapsed();
    let seq_read_throughput = num_elements as f64 / seq_read_duration.as_secs_f64();

    // 랜덤 쓰기
    let start = Instant::now();
    (0..num_elements).into_par_iter().for_each_init(
        || (StdRng::from_entropy(), Uniform::from(0..num_elements)),
        |(rng, dist), _| {
            let idx = dist.sample(rng);
            data[idx].store(rng.gen(), Ordering::SeqCst);
        },
    );
    let rand_write_duration = start.elapsed();
    let rand_write_throughput = num_elements as f64 / rand_write_duration.as_secs_f64();

    // 랜덤 읽기
    let start = Instant::now();
    let sum_rand_read: u64 = (0..num_elements).into_par_iter().map_init(
        || (StdRng::from_entropy(), Uniform::from(0..num_elements)),
        |(rng, dist), _| {
            let idx = dist.sample(rng);
            data[idx].load(Ordering::SeqCst) as u64
        },
    ).sum();
    let rand_read_duration = start.elapsed();
    let rand_read_throughput = num_elements as f64 / rand_read_duration.as_secs_f64();

    println!("순차 쓰기 처리량: {:.2} MB/s", seq_write_throughput / 1_000_000.0);
    println!("순차 읽기 처리량: {:.2} MB/s", seq_read_throughput / 1_000_000.0);
    println!("랜덤 쓰기 처리량: {:.2} MB/s", rand_write_throughput / 1_000_000.0);
    println!("랜덤 읽기 처리량: {:.2} MB/s", rand_read_throughput / 1_000_000.0);
    println!("순차 읽기 합: {}", sum_seq_read);
    println!("랜덤 읽기 합: {}", sum_rand_read);
}

