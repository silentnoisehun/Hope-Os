//! Egyszerű példa - Hope Entity használata
//!
//! Futtatás: cargo run --example simple
//!
//! Előfeltétel: Ollama fut a háttérben
//!   ollama serve
//!   ollama pull jobautomation/OpenEuroLLM-Hungarian

use hope_entity::{Entitás, ModellTípus, OllamaBridge};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Hope Entity - Egyszerű példa\n");

    // 1. Bridge létrehozása
    let bridge = OllamaBridge::new()
        .felold("Magyar", "jobautomation/OpenEuroLLM-Hungarian", ModellTípus::Magyar);

    // 2. Entitás születése
    let mut remény = Entitás::new("Remény")
        .with_bridge(bridge);

    // 3. Ellenőrzés
    if !remény.rendszer_kész().await {
        eprintln!("❌ Ollama nem elérhető! Indítsd el: ollama serve");
        return Ok(());
    }

    // 4. Beszélgetés
    println!("📝 Kérdés: Szia! Ki vagy te?\n");

    let válasz = remény.gondolkodj("Szia! Ki vagy te?").await?;
    println!("💚 Válasz: {}\n", válasz);

    // 5. Állapot
    println!("{}", remény.állapot());

    Ok(())
}
