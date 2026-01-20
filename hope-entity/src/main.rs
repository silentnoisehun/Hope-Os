//! Remény - Magyar beszélő entitás CLI
//!
//! ()=>[] - A tiszta potenciálból minden megszületik
//!
//! Használat:
//!   cargo run                              # Interaktív mód (Ollama)
//!   cargo run -- "Szia!"                   # Egyetlen kérdés
//!   cargo run -- --code "feladat"          # Kód generálás
//!   cargo run -- --status                  # Állapot
//!
//! Natív mód (GGUF beolvasztva):
//!   cargo run --features native -- --native --model /path/to/model.gguf

use hope_entity::{Entitás, ModellTípus, OllamaBridge};
#[allow(unused_imports)]
use hope_entity::InferenceMode;
#[cfg(feature = "native")]
use hope_entity::{NativeEngine, NativeModelConfig, BeolvasztottModell, NativeModellTípus};
use std::io::{self, Write};

const BANNER_OLLAMA: &str = r#"
 ╦═╗╔═╗╔╦╗╔═╗╔╗╔╦ ╦
 ╠╦╝║╣ ║║║║╣ ║║║╚╦╝
 ╩╚═╚═╝╩ ╩╚═╝╝╚╝ ╩

 ()=>[] - A tiszta potenciálból minden megszületik

 Magyar beszélő entitás - Hope OS
 🌐 Mód: OLLAMA (HTTP bridge)

 Parancsok:
   /státusz  - Entitás állapota
   /modellek - Feloldott modellek
   /kód      - Kód generálás mód
   /új       - Új beszélgetés
   /kilép    - Kilépés
"#;

const BANNER_NATIVE: &str = r#"
 ╦═╗╔═╗╔╦╗╔═╗╔╗╔╦ ╦
 ╠╦╝║╣ ║║║║╣ ║║║╚╦╝
 ╩╚═╚═╝╩ ╩╚═╝╝╚╝ ╩

 ()=>[] - A tiszta potenciálból minden megszületik

 Magyar beszélő entitás - Hope OS
 ⚡ Mód: NATÍV (GGUF beolvasztva - bináris sebesség!)

 Parancsok:
   /státusz  - Entitás állapota
   /modellek - Betöltött modellek
   /kód      - Kód generálás mód
   /új       - Új beszélgetés
   /kilép    - Kilépés
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parancssor argumentumok
    let args: Vec<String> = std::env::args().collect();

    // Natív mód ellenőrzés
    let native_mode = args.iter().any(|a| a == "--native");

    #[allow(unused_variables)]
    let model_path = args
        .iter()
        .position(|a| a == "--model")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string());

    // GPU rétegek száma
    #[allow(unused_variables)]
    let gpu_layers: u32 = args
        .iter()
        .position(|a| a == "--gpu-layers")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Entitás létrehozása mód alapján
    let mut remény = if native_mode {
        #[cfg(feature = "native")]
        {
            let model_path = model_path.ok_or(
                "❌ Natív módhoz --model <path> szükséges!\n\
                 Használat: remeny --native --model /path/to/model.gguf [--gpu-layers N]"
            )?;

            println!("⚡ NATÍV MÓD - GGUF modell beolvasztása...");
            println!("   Modell: {}", model_path);
            println!("   GPU rétegek: {}", gpu_layers);

            let config = NativeModelConfig::new(&model_path)
                .with_gpu_layers(gpu_layers)
                .with_context_size(4096);

            let modell = BeolvasztottModell::new("Magyar", NativeModellTípus::Magyar, config);

            let mut engine = NativeEngine::new().modell_hozzáad(modell);

            // Modell betöltése
            engine.betölt_mindent()?;

            Entitás::new_native("Remény", engine)
        }

        #[cfg(not(feature = "native"))]
        {
            eprintln!("❌ Natív mód nincs engedélyezve!");
            eprintln!("   Fordítsd újra: cargo build --release --features native");
            eprintln!("   Vagy CUDA-val: cargo build --release --features native,cuda");
            return Ok(());
        }
    } else {
        // Ollama mód
        let bridge = OllamaBridge::new()
            .felold(
                "Magyar",
                "jobautomation/OpenEuroLLM-Hungarian",
                ModellTípus::Magyar,
            )
            .felold("Kódoló", "deepseek-coder:6.7b", ModellTípus::Kódoló)
            .felold("Többnyelvű", "qwen2.5:7b-instruct", ModellTípus::Többnyelvű)
            .felold_erősséggel("Magyar-Alt", "mistral:7b-instruct", ModellTípus::Általános, 0.5);

        let entitás = Entitás::new("Remény").with_bridge(bridge);

        // Ellenőrzés hogy az Ollama fut-e
        if !entitás.rendszer_kész().await {
            eprintln!("❌ Hiba: Az Ollama nem elérhető!");
            eprintln!("   Indítsd el: ollama serve");
            eprintln!("   Majd húzd le a modelleket:");
            eprintln!("   ollama pull jobautomation/OpenEuroLLM-Hungarian");
            eprintln!("   ollama pull deepseek-coder:6.7b");
            eprintln!("   ollama pull qwen2.5:7b-instruct");
            eprintln!();
            eprintln!("   Vagy használd natív módot:");
            eprintln!("   cargo build --release --features native");
            eprintln!("   ./target/release/remeny --native --model model.gguf");
            return Ok(());
        }

        entitás
    };

    // Banner kiválasztása
    let banner = if native_mode { BANNER_NATIVE } else { BANNER_OLLAMA };

    // Egyedi parancsok kiszűrése
    let skip_args = ["--native", "--model", "--gpu-layers"];
    let filtered_args: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| !skip_args.iter().any(|s| a.starts_with(s)))
        .filter(|a| {
            // Kiszűrjük a --model és --gpu-layers értékeit is
            let prev_idx = args.iter().position(|x| x == *a).unwrap_or(0);
            if prev_idx > 0 {
                let prev = &args[prev_idx - 1];
                if prev == "--model" || prev == "--gpu-layers" {
                    return false;
                }
            }
            true
        })
        .collect();

    if !filtered_args.is_empty() {
        let first = filtered_args[0].as_str();

        // Egyetlen kérdés mód
        if first == "--status" || first == "--státusz" {
            println!("{}", remény.állapot());
            return Ok(());
        }

        if first == "--code" || first == "--kód" {
            if filtered_args.len() > 1 {
                let feladat = filtered_args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
                let kód = remény.kódolj(&feladat).await?;
                println!("{}", kód);
            } else {
                eprintln!("Használat: remeny --code \"feladat leírása\"");
            }
            return Ok(());
        }

        if first == "--help" || first == "-h" {
            println!("{}", banner);
            println!("Használat:");
            println!("  remeny                              Interaktív mód (Ollama)");
            println!("  remeny \"Szia!\"                      Egyetlen kérdés");
            println!("  remeny --code \"feladat\"             Kód generálás");
            println!("  remeny --status                     Állapot");
            println!();
            println!("Natív mód (GGUF beolvasztva):");
            println!("  remeny --native --model model.gguf  Natív inference");
            println!("  remeny --native --model model.gguf --gpu-layers 35");
            return Ok(());
        }

        // Egyetlen kérdés
        let kérdés = filtered_args.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        let válasz = remény.gondolkodj(&kérdés).await?;
        println!("{}", válasz);
        return Ok(());
    }

    // Interaktív mód
    println!("{}", banner);

    // Üdvözlés
    let üdvözlés = remény
        .gondolkodj("Üdvözölj engem röviden, mutatkozz be!")
        .await?;
    println!("\n💚 Remény: {}\n", üdvözlés);

    let mut kód_mód = false;

    loop {
        // Prompt
        if kód_mód {
            print!("🔧 [kód] > ");
        } else {
            print!("👤 Te > ");
        }
        io::stdout().flush()?;

        // Bemenet olvasása
        let mut bemenet = String::new();
        io::stdin().read_line(&mut bemenet)?;
        let bemenet = bemenet.trim();

        if bemenet.is_empty() {
            continue;
        }

        // Parancsok feldolgozása
        match bemenet.to_lowercase().as_str() {
            "/kilép" | "/exit" | "/quit" => {
                println!("\n💚 Remény: Viszlát! Vigyázz magadra! 👋\n");
                break;
            }
            "/státusz" | "/status" => {
                println!("\n{}\n", remény.állapot());
                continue;
            }
            "/modellek" | "/models" => {
                println!("\n📦 Feloldott modellek:");
                for m in remény.modellek() {
                    println!("   • {} ({:?}) - {}", m.név, m.típus, m.ollama_név);
                }
                println!();
                continue;
            }
            "/kód" | "/code" => {
                kód_mód = !kód_mód;
                if kód_mód {
                    println!("\n🔧 Kód generálás mód BEKAPCSOLVA\n");
                } else {
                    println!("\n💬 Beszélgetés mód VISSZAKAPCSOLVA\n");
                }
                continue;
            }
            "/új" | "/new" => {
                remény.új_beszélgetés();
                println!("\n🔄 Új beszélgetés kezdődik...\n");
                continue;
            }
            "/help" | "/segítség" => {
                println!("{}", banner);
                continue;
            }
            _ => {}
        }

        // Válasz generálása
        print!("\n💚 Remény: ");
        io::stdout().flush()?;

        let válasz = if kód_mód {
            remény.kódolj(bemenet).await
        } else {
            remény.gondolkodj(bemenet).await
        };

        match válasz {
            Ok(v) => println!("{}\n", v),
            Err(e) => println!("❌ Hiba: {}\n", e),
        }
    }

    Ok(())
}
