//! Hope OS Benchmark
//!
//! Quick performance test for core operations.

use std::time::Instant;

fn main() {
    println!("Hope OS Benchmark");
    println!("====================\n");

    let iterations = 10_000;

    // Test HashMap operations (simulating memory store)
    let start = Instant::now();
    let mut map = std::collections::HashMap::new();
    for i in 0..iterations {
        map.insert(format!("key_{}", i), format!("value_{}", i));
    }
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!(
        "Memory Store:  {:>10.0} ops/sec ({:?} for {} ops)",
        ops_per_sec, duration, iterations
    );

    // Test HashMap lookup
    let start = Instant::now();
    for i in 0..iterations {
        let _ = map.get(&format!("key_{}", i));
    }
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!(
        "Memory Recall: {:>10.0} ops/sec ({:?} for {} ops)",
        ops_per_sec, duration, iterations
    );

    // Test Vec operations (simulating graph)
    let start = Instant::now();
    let mut vec = Vec::new();
    for i in 0..iterations {
        vec.push(i);
    }
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!(
        "Graph Add:     {:>10.0} ops/sec ({:?} for {} ops)",
        ops_per_sec, duration, iterations
    );

    // Simple math (simulating emotion calculation)
    let start = Instant::now();
    let mut sum = 0.0f64;
    for i in 0..iterations {
        sum += (i as f64 * 0.001).sin();
    }
    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    println!(
        "Emotion Calc:  {:>10.0} ops/sec ({:?} for {} ops)",
        ops_per_sec, duration, iterations
    );

    // Prevent optimization
    if sum.abs() < 0.0 {
        println!("Never printed");
    }

    println!("\nBenchmark complete!");
    println!("()=>[] - From pure potential, everything is born");
}
