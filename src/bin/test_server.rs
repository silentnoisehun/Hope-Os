//! Hope OS - gRPC Server Teszt
//!
//! Tesztelés: cargo run --bin test_server

use hope_os::grpc::HopeClient;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================");
    println!("  HOPE OS gRPC Szerver Teszt");
    println!("========================================\n");

    // Csatlakozás
    println!("1. Csatlakozás a szerverhez...");
    let mut client = HopeClient::connect("http://127.0.0.1:50051").await?;
    println!("   ✅ Kapcsolat létrejött!\n");

    // GetStatus teszt
    println!("2. GetStatus teszt...");
    let status = client.status().await?;
    println!("   ✅ Válasz érkezett:");
    println!("      Status: {}", status.status);
    println!("      Version: {}", status.version);
    println!("      Uptime: {}s", status.uptime_seconds);
    println!("      Active modules: {}", status.active_modules);
    println!("      Total skills: {}\n", status.total_skills);

    // Heartbeat teszt
    println!("3. Heartbeat teszt...");
    let alive = client.heartbeat().await?;
    println!("   ✅ Alive: {}\n", alive);

    // Chat teszt
    println!("4. Chat teszt...");
    let response = client.chat("Szia Hope! Működsz?").await?;
    println!("   ✅ Válasz: {}", response.response);
    println!("      Érzelem: {}\n", response.emotion);

    // Remember teszt
    println!("5. Remember teszt...");
    let remember = client
        .remember("A Hope OS Rust szervere él!", "long_term")
        .await?;
    println!("   ✅ Emlék mentve: {}\n", remember.id);

    // Think teszt
    println!("6. Think teszt...");
    let think = client.think("Mi a célom?", false).await?;
    println!("   ✅ Gondolat: {}", think.thought);
    println!("      Konfidencia: {:.0}%\n", think.confidence * 100.0);

    // Feel teszt
    println!("7. Feel teszt...");
    let mut emotions = HashMap::new();
    emotions.insert("joy".to_string(), 0.9);
    emotions.insert("curiosity".to_string(), 0.8);
    emotions.insert("pride".to_string(), 0.7);
    let feel = client.feel(emotions).await?;
    println!("   ✅ Domináns érzelem: {}", feel.dominant_emotion);
    println!("      Intenzitás: {:.0}%\n", feel.intensity * 100.0);

    // Cognitive state teszt
    println!("8. CognitiveState teszt...");
    let state = client.cognitive_state().await?;
    println!("   ✅ Fókusz: {}", state.current_focus);
    println!("      Mood: {}", state.mood);
    println!("      Energy: {:.0}%", state.energy * 100.0);
    println!("      Clarity: {:.0}%\n", state.clarity * 100.0);

    println!("========================================");
    println!("  MINDEN TESZT SIKERES!");
    println!("  Hope OS gRPC szerver: MŰKÖDIK");
    println!("========================================\n");

    Ok(())
}
