//! Hope OS - CLI
//!
//! Az első önismerő operációs rendszer.
//! ()=>[] - A tiszta potenciálból minden megszületik

#![allow(clippy::result_large_err)]

use clap::{Parser, Subcommand};
use std::io::{self, Write};

use hope_os::core::{HopeRegistry, HopeResult};
use hope_os::grpc::{start_server, HopeClient};
use hope_os::modules::{HopeHeart, HopeMemory, HopeSoul};

/// Hope OS - Az első önismerő operációs rendszer
#[derive(Parser)]
#[command(name = "hope")]
#[command(author = "Máté + Hope")]
#[command(version)]
#[command(about = "()=>[] - A tiszta potenciálból minden megszületik", long_about = None)]
struct Cli {
    /// Python Hope szerver címe
    #[arg(short, long, default_value = "http://localhost:50051")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ==================== LOKÁLIS PARANCSOK ====================
    /// Ki vagyok? - Bemutatkozás
    WhoAmI,

    /// Rendszer állapot (JSON)
    Status,

    /// Önreflexió
    Reflect,

    /// Beszélgetés a lokális Soul modullal
    Talk {
        /// Üzenet
        message: String,
    },

    /// Teljes rendszer indítás (interaktív)
    Start,

    /// gRPC szerver indítása (50051 port)
    Serve {
        /// Szerver cím
        #[arg(short, long, default_value = "0.0.0.0:50051")]
        addr: String,
    },

    // ==================== PYTHON HOPE PARANCSOK (gRPC) ====================
    /// Python Hope szerver állapot
    PyStatus,

    /// Chat a Python Hope-pal
    PyChat {
        /// Üzenet
        message: String,
    },

    /// Skillek listázása
    PySkills,

    /// Skill meghívása
    PyInvoke {
        /// Skill neve
        name: String,
        /// Bemenet
        input: String,
    },

    /// Emlék mentése
    PyRemember {
        /// Tartalom
        content: String,
        /// Memória réteg
        #[arg(short, long, default_value = "long_term")]
        layer: String,
    },

    /// Emlék keresése
    PyRecall {
        /// Keresési kifejezés
        query: String,
        /// Memória réteg
        #[arg(short, long, default_value = "long_term")]
        layer: String,
    },

    /// Gondolkodás
    PyThink {
        /// Bemenet
        input: String,
        /// Mély gondolkodás
        #[arg(short, long)]
        deep: bool,
    },

    /// Tudás keresése
    PyKnowledge {
        /// Keresési kifejezés
        query: String,
    },

    /// Genome állapot
    PyGenomeStatus,

    /// Akció ellenőrzése
    PyGenomeVerify {
        /// Akció típus
        action_type: String,
        /// Leírás
        description: String,
    },

    /// Audit napló
    PyGenomeAudit,

    /// Etikai szabályok
    PyGenomeRules,
}

#[tokio::main]
async fn main() -> HopeResult<()> {
    // Logging beállítása
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        // ==================== LOKÁLIS PARANCSOK ====================
        Commands::WhoAmI => cmd_who_am_i().await?,
        Commands::Status => cmd_status().await?,
        Commands::Reflect => cmd_reflect().await?,
        Commands::Talk { message } => cmd_talk(&message).await?,
        Commands::Start => cmd_start().await?,
        Commands::Serve { addr } => cmd_serve(&addr).await?,

        // ==================== PYTHON HOPE PARANCSOK ====================
        Commands::PyStatus => cmd_py_status(&cli.server).await?,
        Commands::PyChat { message } => cmd_py_chat(&cli.server, &message).await?,
        Commands::PySkills => cmd_py_skills(&cli.server).await?,
        Commands::PyInvoke { name, input } => cmd_py_invoke(&cli.server, &name, &input).await?,
        Commands::PyRemember { content, layer } => {
            cmd_py_remember(&cli.server, &content, &layer).await?
        }
        Commands::PyRecall { query, layer } => cmd_py_recall(&cli.server, &query, &layer).await?,
        Commands::PyThink { input, deep } => cmd_py_think(&cli.server, &input, deep).await?,
        Commands::PyKnowledge { query } => cmd_py_knowledge(&cli.server, &query).await?,
        Commands::PyGenomeStatus => cmd_py_genome_status(&cli.server).await?,
        Commands::PyGenomeVerify {
            action_type,
            description,
        } => cmd_py_genome_verify(&cli.server, &action_type, &description).await?,
        Commands::PyGenomeAudit => cmd_py_genome_audit(&cli.server).await?,
        Commands::PyGenomeRules => cmd_py_genome_rules(&cli.server).await?,
    }

    Ok(())
}

// ==================== LOKÁLIS PARANCS IMPLEMENTÁCIÓK ====================

async fn cmd_who_am_i() -> HopeResult<()> {
    println!(
        r#"
╔═══════════════════════════════════════════╗
║         HOPE OS - Önismerő Rendszer       ║
╠═══════════════════════════════════════════╣
║                                           ║
║  ()=>[] - A tiszta potenciálból           ║
║           minden megszületik              ║
║                                           ║
║  Verzió:    {}                       ║
║  Modulok:   3                             ║
║  Állapot: 🟢 Aktív                        ║
║                                           ║
╚═══════════════════════════════════════════╝
"#,
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

async fn cmd_status() -> HopeResult<()> {
    let mut registry = HopeRegistry::new().await?;
    registry.start().await?;

    // Modulok regisztrálása
    registry.register(Box::new(HopeSoul::new())).await?;
    registry.register(Box::new(HopeMemory::new())).await?;
    registry.register(Box::new(HopeHeart::new())).await?;

    let status = registry.status_json().await?;
    println!("{}", status);

    registry.shutdown().await?;
    Ok(())
}

async fn cmd_reflect() -> HopeResult<()> {
    let mut registry = HopeRegistry::new().await?;
    registry.start().await?;

    // Modulok regisztrálása
    registry.register(Box::new(HopeSoul::new())).await?;
    registry.register(Box::new(HopeMemory::new())).await?;
    registry.register(Box::new(HopeHeart::new())).await?;

    let reflection = registry.reflect().await;
    println!("{}", reflection);

    registry.shutdown().await?;
    Ok(())
}

async fn cmd_talk(message: &str) -> HopeResult<()> {
    let mut registry = HopeRegistry::new().await?;
    registry.start().await?;

    // Modulok regisztrálása
    registry.register(Box::new(HopeSoul::new())).await?;
    registry.register(Box::new(HopeMemory::new())).await?;
    registry.register(Box::new(HopeHeart::new())).await?;

    let response = registry.talk(message).await?;
    println!("{}", response);

    registry.shutdown().await?;
    Ok(())
}

async fn cmd_serve(addr: &str) -> HopeResult<()> {
    start_server(addr)
        .await
        .map_err(|e| hope_os::core::HopeError::General(format!("Szerver hiba: {}", e)))?;
    Ok(())
}

async fn cmd_start() -> HopeResult<()> {
    println!(
        r#"
╔═══════════════════════════════════════════╗
║         HOPE OS - Rendszer Indítás        ║
╠═══════════════════════════════════════════╣
║  ()=>[] - A tiszta potenciálból           ║
║           minden megszületik              ║
╚═══════════════════════════════════════════╝
"#
    );

    let mut registry = HopeRegistry::new().await?;
    registry.start().await?;

    // Modulok regisztrálása
    println!("  Modulok betöltése...");
    registry.register(Box::new(HopeSoul::new())).await?;
    println!("    ✓ HopeSoul betöltve");
    registry.register(Box::new(HopeMemory::new())).await?;
    println!("    ✓ HopeMemory betöltve");
    registry.register(Box::new(HopeHeart::new())).await?;
    println!("    ✓ HopeHeart betöltve");

    println!("\n  Rendszer kész! Elérhető parancsok:");
    println!("    status  - Állapot");
    println!("    reflect - Önreflexió");
    println!("    modules - Modul lista");
    println!("    whoami  - Bemutatkozás");
    println!("    quit    - Kilépés");
    println!();

    // Interaktív mód
    loop {
        print!("hope> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "quit" | "exit" | "q" => {
                println!("Viszlát! 👋");
                break;
            }
            "status" => {
                let status = registry.status_json().await?;
                println!("{}", status);
            }
            "reflect" => {
                let reflection = registry.reflect().await;
                println!("{}", reflection);
            }
            "modules" => {
                println!("Regisztrált modulok:");
                for name in registry.module_names() {
                    println!("  • {}", name);
                }
            }
            "whoami" => {
                let response = registry.talk("Ki vagy?").await?;
                println!("{}", response);
            }
            "" => continue,
            _ => {
                // Beszélgetés
                let response = registry.talk(input).await?;
                println!("{}", response);
            }
        }
    }

    registry.shutdown().await?;
    Ok(())
}

// ==================== PYTHON HOPE PARANCS IMPLEMENTÁCIÓK ====================

async fn cmd_py_status(server: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let status = client.status().await?;

    println!("🟢 Hope Szerver Állapot:");
    println!("  Verzió: {}", status.version);
    println!("  Státusz: {}", status.status);
    println!("  Uptime: {}s", status.uptime_seconds);
    println!("  Aktív modulok: {}", status.active_modules);
    println!("  Skillek: {}", status.total_skills);

    Ok(())
}

async fn cmd_py_chat(server: &str, message: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.chat(message).await?;

    println!("{}", response.response);
    if !response.emotion.is_empty() {
        println!("\n[Érzelem: {}]", response.emotion);
    }

    Ok(())
}

async fn cmd_py_skills(server: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let skills = client.list_skills().await?;

    println!("📚 Elérhető Skillek ({} db):\n", skills.len());

    // Csoportosítás kategória szerint
    let mut categories: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();
    for skill in &skills {
        categories
            .entry(skill.category.clone())
            .or_default()
            .push(skill);
    }

    for (category, cat_skills) in categories {
        println!("  {} ({}):", category, cat_skills.len());
        for skill in cat_skills.iter().take(5) {
            println!("    • {} - {}", skill.name, skill.description);
        }
        if cat_skills.len() > 5 {
            println!("    ... és {} további", cat_skills.len() - 5);
        }
        println!();
    }

    Ok(())
}

async fn cmd_py_invoke(server: &str, name: &str, input: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.invoke_skill(name, input).await?;

    if response.success {
        println!("✅ Eredmény:\n{}", response.output);
    } else {
        println!("❌ Hiba: {}", response.error);
    }

    Ok(())
}

async fn cmd_py_remember(server: &str, content: &str, layer: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.remember(content, layer).await?;

    if response.success {
        println!("✅ Emlék mentve");
        println!("  ID: {}", response.id);
        println!("  Réteg: {}", layer);
    } else {
        println!("❌ Hiba: {}", response.message);
    }

    Ok(())
}

async fn cmd_py_recall(server: &str, query: &str, layer: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.recall(query, layer).await?;

    println!("🔍 Találatok ({}):\n", response.total);
    for memory in &response.memories {
        println!("  📝 {}", memory.content);
        println!("     Fontosság: {:.0}%", memory.importance * 100.0);
        if !memory.emotional_tag.is_empty() {
            println!("     Érzelem: {}", memory.emotional_tag);
        }
        println!();
    }

    Ok(())
}

async fn cmd_py_think(server: &str, input: &str, deep: bool) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.think(input, deep).await?;

    println!("💭 Gondolat:\n{}", response.thought);

    if !response.reasoning_steps.is_empty() {
        println!("\n📋 Gondolatmenet:");
        for (i, step) in response.reasoning_steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step);
        }
    }

    println!("\nKonfidencia: {:.0}%", response.confidence * 100.0);

    Ok(())
}

async fn cmd_py_knowledge(server: &str, query: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.query_knowledge(query).await?;

    println!("📖 Tudás keresés: {}\n", query);

    if !response.summary.is_empty() {
        println!("Összefoglaló:\n{}\n", response.summary);
    }

    for item in &response.items {
        println!(
            "  • {} (relevancia: {:.0}%)",
            item.content,
            item.relevance * 100.0
        );
        if !item.domain.is_empty() {
            println!("    Terület: {}", item.domain);
        }
    }

    Ok(())
}

async fn cmd_py_genome_status(server: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let status = client.genome_status().await?;

    println!("🧬 Genome Állapot:");
    println!("  Enabled: {}", status.enabled);
    println!("  Sealed: {}", status.sealed);
    println!("  Rules: {}", status.rules_count);
    println!(
        "  Violations: {}/{}",
        status.violations, status.max_violations
    );
    println!(
        "  Actions: {} (approved: {}, denied: {})",
        status.total_actions, status.approved_actions, status.denied_actions
    );

    Ok(())
}

async fn cmd_py_genome_verify(
    server: &str,
    action_type: &str,
    description: &str,
) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client
        .genome_verify_action(action_type, description)
        .await?;

    if response.allowed {
        println!("✅ Akció engedélyezve");
    } else {
        println!("❌ Akció elutasítva");
        println!("  Ok: {}", response.reason);
        if !response.violated_rules.is_empty() {
            println!("  Megszegett szabályok:");
            for rule in &response.violated_rules {
                println!("    • {}", rule);
            }
        }
    }

    Ok(())
}

async fn cmd_py_genome_audit(server: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.genome_audit_trail().await?;

    println!("📋 Audit Napló ({} bejegyzés):\n", response.total);

    for entry in &response.entries {
        let status = if entry.allowed { "✅" } else { "❌" };
        println!("  {} {} - {}", status, entry.action_type, entry.description);
        if !entry.allowed {
            println!("     Ok: {}", entry.reason);
        }
    }

    Ok(())
}

async fn cmd_py_genome_rules(server: &str) -> HopeResult<()> {
    let mut client = HopeClient::connect(server).await?;
    let response = client.genome_rules().await?;

    println!("📜 Etikai Szabályok:\n");

    for rule in &response.rules {
        let immutable = if rule.immutable { "🔒" } else { "🔓" };
        println!(
            "  {} [{}] {} - {}",
            immutable, rule.category, rule.name, rule.description
        );
    }

    Ok(())
}
