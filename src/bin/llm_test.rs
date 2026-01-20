//! Hope OS - LLM Integration Test
//!
//! Teszt a CodeGraph és NeuroBlast funkcionalitáshoz.
//! ()=>[] - A tiszta potenciálból minden megszületik

#![allow(clippy::result_large_err)]

use hope_os::data::{BlockType, CodeBlock, CodeGraph, ConnectionType, NeuroGraph, WaveType};

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     HOPE OS - LLM INTEGRATION TEST                        ║");
    println!("║     ()=>[] - A tiszta potenciálból minden megszületik     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // 1. CodeGraph teszt
    println!("=== 1. CODEGRAPH TESZT ===\n");

    let graph = CodeGraph::new();

    // Emlékek hozzáadása
    let mem1 = graph.remember("Az LLM tesztelése fontos", 0.9).unwrap();
    let mem2 = graph.remember("A Hope OS önismerő rendszer", 0.95).unwrap();
    let mem3 = graph.remember("A kód maga a gráf", 1.0).unwrap();

    println!("✓ 3 emlék létrehozva");

    // Érzelmek
    let emo1 = graph.feel("curiosity", 0.8, Some("LLM teszt")).unwrap();
    let _emo2 = graph.feel("joy", 0.7, Some("sikeres teszt")).unwrap();

    println!("✓ 2 érzelem rögzítve");

    // Kapcsolatok
    graph.connect(&mem1, &mem2, ConnectionType::AssociatesWith, 0.8);
    graph.connect(&mem2, &mem3, ConnectionType::DependsOn, 0.9);
    graph.connect(&emo1, &mem1, ConnectionType::Triggers, 0.7);

    println!("✓ 3 kapcsolat létrehozva");

    // Keresés
    let found = graph.search("LLM");
    println!("✓ Keresés 'LLM': {} találat", found.len());

    // Statisztika
    let stats = graph.stats();
    println!("\nCodeGraph statisztika:");
    println!("  Blokkok: {}", stats.total_blocks);
    println!("  Kapcsolatok: {}", stats.total_connections);
    println!(
        "  Típusok: Memory={}, Emotion={}",
        stats.type_counts.get(&BlockType::Memory).unwrap_or(&0),
        stats.type_counts.get(&BlockType::Emotion).unwrap_or(&0)
    );

    // 2. NeuroBlast teszt
    println!("\n=== 2. NEUROBLAST TESZT ===\n");

    let neuro = NeuroGraph::new();

    // Neuronok (blokkok) hozzáadása
    let n1 = neuro
        .add_block(
            CodeBlock::new(
                "input_node",
                "Bemeneti neuron",
                BlockType::Data,
                "LLM input",
            )
            .with_importance(0.9),
        )
        .unwrap();

    let n2 = neuro
        .add_block(
            CodeBlock::new(
                "hidden_node",
                "Rejtett neuron",
                BlockType::Function,
                "Processing",
            )
            .with_importance(0.7),
        )
        .unwrap();

    let n3 = neuro
        .add_block(
            CodeBlock::new(
                "output_node",
                "Kimeneti neuron",
                BlockType::Data,
                "LLM output",
            )
            .with_importance(0.9),
        )
        .unwrap();

    println!("✓ 3 neuron létrehozva");

    // Szinaptikus kapcsolatok
    neuro.graph.connect(&n1, &n2, ConnectionType::Triggers, 0.8);
    neuro.graph.connect(&n2, &n3, ConnectionType::Triggers, 0.9);

    println!("✓ 2 szinaptikus kapcsolat");

    // Hullám indítása
    let wave_id = neuro.emit_wave(&n1, "test_signal", WaveType::Impulse);
    println!("✓ Impulzus hullám elindítva: {:?}", wave_id.is_some());

    // Szimuláció futtatása
    let results = neuro.run_until_calm(5);
    println!("✓ Szimuláció: {} tick futott", results.len());

    // Eredmények
    if let Some(last) = results.last() {
        println!("\nNeuroBlast eredmények:");
        println!("  Aktív hullámok: {}", last.active_waves);
        println!("  Propagációk: {}", last.propagations);
        println!("  Neuron tüzelések: {}", last.neurons_fired);
    }

    // 3. Gondolat teszt
    println!("\n=== 3. GONDOLAT TESZT ===\n");

    let thought_wave = neuro.think("Az LLM integráció sikeres!", 0.9);
    println!("✓ Gondolat hullám: {:?}", thought_wave.is_some());

    // Még egy tick
    let result = neuro.tick();
    println!(
        "✓ Tick #{}: {} aktív hullám",
        result.tick, result.active_waves
    );

    // 4. Végső statisztika
    println!("\n=== VÉGSŐ STATISZTIKA ===\n");

    let neuro_stats = neuro.stats();
    println!("NeuroGraph:");
    println!("  Neuronok: {}", neuro_stats.total_neurons);
    println!("  Aktív hullámok: {}", neuro_stats.active_waves);
    println!("  Interferenciák: {}", neuro_stats.total_interferences);
    println!("  Tick: {}", neuro_stats.current_tick);

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║     LLM INTEGRATION TEST: SIKERES ✓                       ║");
    println!("║     A KÓD MAGA A GRÁF - NINCS KÜLSŐ DB!                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
}
