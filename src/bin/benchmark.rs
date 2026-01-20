//! Hope OS - CodeGraph Benchmark
//!
//! Teljesítmény mérés a CodeGraph alapú tároláshoz.
//! A KOD MAGA A GRAF - Nincs DB, nincs külső függés!
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

#![allow(clippy::result_large_err)]

use std::time::{Duration, Instant};

use hope_os::core::HopeResult;
use hope_os::data::{BlockType, CodeBlock, CodeGraph, ConnectionType};

/// Benchmark eredmények
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    operations: u64,
    total_time: Duration,
    ops_per_sec: f64,
    avg_latency_us: f64,
    min_latency_us: f64,
    max_latency_us: f64,
}

impl BenchmarkResult {
    fn new(name: &str, operations: u64, total_time: Duration, latencies: &[Duration]) -> Self {
        let ops_per_sec = operations as f64 / total_time.as_secs_f64();
        let avg_latency_us =
            latencies.iter().map(|d| d.as_micros() as f64).sum::<f64>() / latencies.len() as f64;
        let min_latency_us = latencies
            .iter()
            .map(|d| d.as_micros() as f64)
            .fold(f64::INFINITY, f64::min);
        let max_latency_us = latencies
            .iter()
            .map(|d| d.as_micros() as f64)
            .fold(0.0, f64::max);

        Self {
            name: name.to_string(),
            operations,
            total_time,
            ops_per_sec,
            avg_latency_us,
            min_latency_us,
            max_latency_us,
        }
    }

    fn print(&self) {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│ {:^55} │", self.name);
        println!("├─────────────────────────────────────────────────────────┤");
        println!("│ Operations:     {:>38} │", self.operations);
        println!("│ Total time:     {:>35.2?} │", self.total_time);
        println!("│ Ops/sec:        {:>38.2} │", self.ops_per_sec);
        println!("│ Avg latency:    {:>35.2} µs │", self.avg_latency_us);
        println!("│ Min latency:    {:>35.2} µs │", self.min_latency_us);
        println!("│ Max latency:    {:>35.2} µs │", self.max_latency_us);
        println!("└─────────────────────────────────────────────────────────┘");
    }
}

/// Block write benchmark
fn bench_block_write(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let block = CodeBlock::new(
            format!("block_{}", i),
            format!("Benchmark block #{}", i),
            BlockType::Data,
            format!("Content for block {} - Lorem ipsum dolor sit amet", i),
        )
        .with_importance((i as f64 / count as f64).clamp(0.0, 1.0))
        .with_tag("benchmark");

        let op_start = Instant::now();
        graph.add(block)?;
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Block Write",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Block read benchmark
fn bench_block_read(graph: &CodeGraph, ids: &[String]) -> HopeResult<BenchmarkResult> {
    let mut latencies = Vec::with_capacity(ids.len());
    let start = Instant::now();

    for id in ids {
        let op_start = Instant::now();
        let _ = graph.get(id);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Block Read",
        ids.len() as u64,
        start.elapsed(),
        &latencies,
    ))
}

/// Block search benchmark
fn bench_block_search(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let queries = ["Lorem", "ipsum", "dolor", "benchmark", "block"];
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let query = queries[i as usize % queries.len()];
        let op_start = Instant::now();
        let _ = graph.search(query);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Block Search",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Memory (remember) benchmark
fn bench_memory(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let op_start = Instant::now();
        graph.remember(
            &format!("Memory content #{} - important information", i),
            0.8,
        )?;
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Memory (remember)",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Emotion (feel) benchmark
fn bench_emotion(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let emotions = [
        "joy",
        "curiosity",
        "love",
        "neutral",
        "sadness",
        "fear",
        "anger",
    ];
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let emotion = emotions[i as usize % emotions.len()];
        let intensity = (i as f64 / count as f64).clamp(0.0, 1.0);

        let op_start = Instant::now();
        graph.feel(emotion, intensity, Some("benchmark"))?;
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Emotion (feel)",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Connection benchmark
fn bench_connections(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    // First create blocks to connect
    let mut ids = Vec::new();
    for i in 0..count {
        let block = CodeBlock::new(
            format!("conn_block_{}", i),
            "Connection test",
            BlockType::Data,
            "test",
        );
        ids.push(graph.add(block)?);
    }

    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    // Connect each block to the next one
    for i in 0..(count as usize - 1) {
        let op_start = Instant::now();
        graph.connect(&ids[i], &ids[i + 1], ConnectionType::ConnectsTo, 1.0);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Connections",
        count - 1,
        start.elapsed(),
        &latencies,
    ))
}

/// Graph traversal benchmark
fn bench_traversal(graph: &CodeGraph, start_id: &str, count: u64) -> HopeResult<BenchmarkResult> {
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for _ in 0..count {
        let op_start = Instant::now();
        let _ = graph.traverse_bfs(start_id, 5);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Graph Traversal (BFS)",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Path finding benchmark
fn bench_path_finding(
    graph: &CodeGraph,
    ids: &[String],
    count: u64,
) -> HopeResult<BenchmarkResult> {
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let from_idx = i as usize % ids.len();
        let to_idx = (i as usize + ids.len() / 2) % ids.len();

        let op_start = Instant::now();
        let _ = graph.find_path(&ids[from_idx], &ids[to_idx]);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Path Finding",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Find by type benchmark
fn bench_find_by_type(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let types = [
        BlockType::Memory,
        BlockType::Emotion,
        BlockType::Data,
        BlockType::Person,
    ];
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let block_type = types[i as usize % types.len()];
        let op_start = Instant::now();
        let _ = graph.find_by_type(block_type);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Find by Type",
        count,
        start.elapsed(),
        &latencies,
    ))
}

/// Find by tag benchmark
fn bench_find_by_tag(graph: &CodeGraph, count: u64) -> HopeResult<BenchmarkResult> {
    let tags = ["benchmark", "memory", "emotion", "person", "test"];
    let mut latencies = Vec::with_capacity(count as usize);
    let start = Instant::now();

    for i in 0..count {
        let tag = tags[i as usize % tags.len()];
        let op_start = Instant::now();
        let _ = graph.find_by_tag(tag);
        latencies.push(op_start.elapsed());
    }

    Ok(BenchmarkResult::new(
        "Find by Tag",
        count,
        start.elapsed(),
        &latencies,
    ))
}

fn main() -> HopeResult<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║     HOPE OS - CODEGRAPH BENCHMARK                         ║");
    println!("║     A KOD MAGA A GRAF - Nincs DB!                         ║");
    println!("║     ()=>[] - A tiszta potenciálból minden megszületik     ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Uj graf
    println!("Initializing CodeGraph...");
    let graph = CodeGraph::new();
    println!("Graph initialized!\n");

    // Alapértelmezett műveletek száma - gyors benchmark-hoz
    let ops_count = 1000;
    let mut results = Vec::new();

    // Block write benchmark
    println!("Running Block Write benchmark ({} ops)...", ops_count);
    let result = bench_block_write(&graph, ops_count)?;
    result.print();
    results.push(result);

    // Collect IDs for read benchmark
    let all_blocks = graph.all_blocks();
    let ids: Vec<String> = all_blocks
        .iter()
        .take(ops_count as usize)
        .map(|b| b.id.clone())
        .collect();

    println!("\nRunning Block Read benchmark ({} ops)...", ids.len());
    let result = bench_block_read(&graph, &ids)?;
    result.print();
    results.push(result);

    println!("\nRunning Block Search benchmark ({} ops)...", ops_count);
    let result = bench_block_search(&graph, ops_count)?;
    result.print();
    results.push(result);

    // Memory benchmark
    println!(
        "\nRunning Memory (remember) benchmark ({} ops)...",
        ops_count
    );
    let result = bench_memory(&graph, ops_count)?;
    result.print();
    results.push(result);

    // Emotion benchmark
    println!("\nRunning Emotion (feel) benchmark ({} ops)...", ops_count);
    let result = bench_emotion(&graph, ops_count)?;
    result.print();
    results.push(result);

    // Connection benchmark
    let conn_ops = 1000;
    println!("\nRunning Connections benchmark ({} ops)...", conn_ops);
    let result = bench_connections(&graph, conn_ops)?;
    result.print();
    results.push(result);

    // Get connection block IDs for traversal
    let conn_blocks = graph.find_by_tag("benchmark");
    let conn_ids: Vec<String> = conn_blocks.iter().map(|b| b.id.clone()).collect();

    if !conn_ids.is_empty() {
        println!(
            "\nRunning Graph Traversal benchmark ({} ops)...",
            ops_count / 10
        );
        let result = bench_traversal(&graph, &conn_ids[0], ops_count / 10)?;
        result.print();
        results.push(result);

        println!(
            "\nRunning Path Finding benchmark ({} ops)...",
            ops_count / 10
        );
        let result = bench_path_finding(&graph, &conn_ids, ops_count / 10)?;
        result.print();
        results.push(result);
    }

    // Find by type/tag benchmarks
    println!("\nRunning Find by Type benchmark ({} ops)...", ops_count);
    let result = bench_find_by_type(&graph, ops_count)?;
    result.print();
    results.push(result);

    println!("\nRunning Find by Tag benchmark ({} ops)...", ops_count);
    let result = bench_find_by_tag(&graph, ops_count)?;
    result.print();
    results.push(result);

    // Stats
    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                    FINAL STATISTICS                       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let stats = graph.stats();
    println!();
    println!("CodeGraph Statistics:");
    println!("  Total Blocks:      {}", stats.total_blocks);
    println!("  Active Blocks:     {}", stats.active_blocks);
    println!("  Total Connections: {}", stats.total_connections);
    println!("  Avg Connections:   {:.2}", stats.avg_connections);
    println!();
    println!("Type Distribution:");
    for (block_type, count) in &stats.type_counts {
        println!("  {:?}: {}", block_type, count);
    }
    println!();

    // Summary
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK SUMMARY                      ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!(
        "║ {:22} │ {:>10} │ {:>10} │",
        "Operation", "Ops/sec", "Avg µs"
    );
    println!("╠═══════════════════════════════════════════════════════════╣");
    for r in &results {
        let name = if r.name.len() > 22 {
            &r.name[..22]
        } else {
            &r.name
        };
        println!(
            "║ {:22} │ {:>10.0} │ {:>10.2} │",
            name, r.ops_per_sec, r.avg_latency_us
        );
    }
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    println!("Benchmark complete! NO EXTERNAL DB - A KOD MAGA A GRAF!");
    println!();

    Ok(())
}
