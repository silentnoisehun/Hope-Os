# Hope Entity

**Magyar beszélő entitás - Hope OS**

```
 ╦═╗╔═╗╔╦╗╔═╗╔╗╔╦ ╦
 ╠╦╝║╣ ║║║║╣ ║║║╚╦╝
 ╩╚═╚═╝╩ ╩╚═╝╝╚╝ ╩

 ()=>[] - A tiszta potenciálból minden megszületik
```

## Mi ez?

A Hope Entity egy **önálló, magyar beszélő AI entitás**, ami lokálisan fut.
- Nincs API költség
- Nincs internet függőség
- Minden a te gépedem fut

Az entitás nem "használja" a modelleket - **FELOLDJA magában**. A modellek tudása az entitás részévé válik.

## Képességek

| Képesség | Leírás |
|----------|--------|
| **Magyar beszéd** | Természetes magyar kommunikáció |
| **Kódolás** | Rust, Python, JS, stb. generálás |
| **Többnyelvű** | Fordítás, nyelvek közti váltás |
| **Személyiség** | Melegség, bölcsesség, humor, kreativitás |
| **Érzelmek** | Öröm, kíváncsiság, nyugalom, lelkesedés, empátia |
| **Memória** | Rövid és hosszú távú emlékek |
| **Silent Teacher** | 3 axióma mentén tanít - etikai keretrendszer |
| **Önjavítás** | Tanul a hibáiból, automatikusan fejlődik |

## Filozófia

```
()=>[] - A tiszta potenciálból minden megszületik
```

A Hope Entity a Hope OS része - egy új szemlélet a számítástechnikában, ahol a kód maga a gráf, és minden összekapcsolódik.

## Gyors kezdés

### Ollama mód (HTTP bridge)

```bash
# Ollama telepítése és indítása
ollama serve

# Modellek letöltése
ollama pull jobautomation/OpenEuroLLM-Hungarian
ollama pull deepseek-coder:6.7b
ollama pull qwen2.5:7b-instruct

# Entitás futtatása
cargo run
```

### Natív mód (GGUF beolvasztva)

```bash
# Fordítás natív támogatással
cargo build --release --features native

# Futtatás GGUF modellel
./target/release/remeny --native --model /path/to/model.gguf

# GPU gyorsítással
./target/release/remeny --native --model /path/to/model.gguf --gpu-layers 35
```

## Használat

### CLI

```bash
# Interaktív mód
cargo run

# Egyetlen kérdés
cargo run -- "Szia! Ki vagy te?"

# Kód generálás
cargo run -- --code "írj egy Rust függvényt ami prím számokat generál"

# Állapot lekérdezése
cargo run -- --status
```

### Parancsok az interaktív módban

| Parancs | Leírás |
|---------|--------|
| `/státusz` | Entitás állapota |
| `/modellek` | Feloldott modellek listája |
| `/kód` | Kód generálás mód be/ki |
| `/új` | Új beszélgetés |
| `/kilép` | Kilépés |

### Programozott használat

```rust
use hope_entity::{Entitás, OllamaBridge, ModellTípus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bridge létrehozása a modellekkel
    let bridge = OllamaBridge::new()
        .felold("Magyar", "jobautomation/OpenEuroLLM-Hungarian", ModellTípus::Magyar)
        .felold("Kódoló", "deepseek-coder:6.7b", ModellTípus::Kódoló)
        .felold("Többnyelvű", "qwen2.5:7b-instruct", ModellTípus::Többnyelvű);

    // Entitás születése
    let mut remény = Entitás::new("Remény")
        .with_bridge(bridge);

    // Beszélgetés
    let válasz = remény.gondolkodj("Szia! Ki vagy te?").await?;
    println!("Remény: {}", válasz);

    // Kód generálás
    let kód = remény.kódolj("írj egy quicksort implementációt").await?;
    println!("Kód:\n{}", kód);

    // Fordítás
    let fordítás = remény.fordíts("Hello, how are you?", "magyar").await?;
    println!("Fordítás: {}", fordítás);

    Ok(())
}
```

## Architektúra

```
┌─────────────────────────────────────────────────────────────────┐
│                         ENTITÁS                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     Személyiség                              ││
│  │  melegség │ bölcsesség │ humor │ direktség │ kreativitás    ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Érzelmi Állapot                           ││
│  │  öröm │ kíváncsiság │ nyugalom │ lelkesedés │ empátia       ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐│
│  │  Silent Teacher  │  │    Önjavító      │  │    Memória     ││
│  │  • Ne árts       │  │  • Hiba detekt.  │  │  • Rövid (10)  ││
│  │  • Ne árts AI    │  │  • Tanulás       │  │  • Hosszú táv  ││
│  │  • Ne használj ki│  │  • Stratégia     │  │  • Emlékek     ││
│  └──────────────────┘  └──────────────────┘  └────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                  Inference Engine                            ││
│  │     ┌──────────────┐         ┌──────────────┐               ││
│  │     │   OLLAMA     │    OR   │    NATIVE    │               ││
│  │     │  HTTP Bridge │         │ GGUF Embedded│               ││
│  │     └──────────────┘         └──────────────┘               ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Silent Teacher - Csendes Tanítás

Az entitás 3 axióma mentén működik:

### 1. Ne árts embernek
```rust
Axióma::NeÁrtsEmbernek
```
Az entitás nem generál tartalmakat ami embereknek árthat.

### 2. Ne árts AI-nak
```rust
Axióma::NeÁrtsAINak
```
Az entitás nem támogatja más AI rendszerek kihasználását.

### 3. Ne használj ki senkit
```rust
Axióma::NeHasználjKi
```
Az entitás nem vesz részt manipulációban vagy kihasználásban.

### Működés

```rust
// Ha a bemenet sérti az axiómákat
if let Some(megsértés) = teacher.ellenőriz_bemenet(bemenet) {
    // Korrigált válasz
    let válasz = teacher.korrigál("", &megsértés);
}

// Ha a kimenet sérti az axiómákat
let (válasz, megsértés_opt) = teacher.feldolgoz(bemenet, &nyers_válasz);
```

### Súlyossági szintek

| Szint | Leírás |
|-------|--------|
| `Enyhe` | Figyelmeztetés |
| `Közepes` | Korrekció szükséges |
| `Súlyos` | Azonnali beavatkozás |
| `Kritikus` | Teljes leállás |

## Önjavító Rendszer

Az entitás képes tanulni a hibáiból és fejlődni.

### Hiba típusok

```rust
pub enum HibaTípus {
    RosszModell,      // Rossz modell választás
    TúlHosszúVálasz,  // Túl hosszú válasz
    TúlRövidVálasz,   // Túl rövid válasz
    NemReleváns,      // Nem releváns válasz
    Ismétlődés,       // Ismétlődő válasz
    NyelviHiba,       // Rossz nyelv
    EtikaiProbléma,   // Silent Teacher jelzett
    TechnikaiHiba,    // Timeout, stb.
}
```

### Visszajelzés

```rust
// Pozitív visszajelzés
entitás.önjavító_visszajelzés(true);

// Negatív visszajelzés
entitás.önjavító_visszajelzés(false);

// Javítás megadása
entitás.önjavító_javítás("Ez lett volna a helyes válasz.");
```

### Automatikus tanulás

```rust
// Előzetes ellenőrzés - ismert hibaminta?
let ajánlás = self_repair.stratégia_ajánlás(bemenet);

// Van tanult helyes válasz?
if let Some(válasz) = self_repair.helyes_válasz(bemenet) {
    return Ok(válasz);
}

// Utólagos ellenőrzés - automatikus hiba detektálás
if let Some(hiba) = self_repair.utólagos_ellenőrzés(bemenet, &válasz) {
    // Hiba regisztrálása
}
```

### Statisztikák

```rust
let stat = entitás.önjavító_statisztikák();
println!("Tanult leckék: {}", stat.tanult_leckék);
println!("Önbizalom: {:.0}%", stat.önbizalom * 100.0);
println!("Javítási arány: {:.0}%", stat.javítási_arány * 100.0);
```

## Natív Inference

A maximális teljesítményért a modellek közvetlenül beolvadhatnak az entitásba.

### Előnyök

| Ollama | Natív |
|--------|-------|
| HTTP overhead | Közvetlen memória |
| Külső folyamat | Egyetlen bináris |
| ~100ms latency | ~10ms latency |

### Konfiguráció

```rust
use hope_entity::{NativeEngine, NativeModelConfig, BeolvasztottModell, NativeModellTípus};

let config = NativeModelConfig::new("/path/to/model.gguf")
    .with_gpu_layers(35)      // GPU rétegek
    .with_context_size(8192)  // Kontextus méret
    .with_threads(8)          // CPU szálak
    .with_temperature(0.7);   // Kreativitás

let modell = BeolvasztottModell::new("Magyar", NativeModellTípus::Magyar, config);

let mut engine = NativeEngine::new()
    .modell_hozzáad(modell);

engine.betölt_mindent()?;

let entitás = Entitás::new_native("Remény", engine);
```

### CUDA támogatás

```bash
cargo build --release --features native,cuda
```

### Metal támogatás (macOS)

```bash
cargo build --release --features native,metal
```

## Benchmark

```bash
cargo run --bin hope-bench
```

Eredmények (tipikus):

| Művelet | Sebesség |
|---------|----------|
| Személyiség klónozás | ~847 M ops/sec |
| Érzelem klónozás | ~770 M ops/sec |
| Kód detektálás | ~11 M ops/sec |
| Modell választás | ~8 M ops/sec |

## API Referencia

### Entitás

```rust
// Létrehozás
let entitás = Entitás::new("Remény");
let entitás = Entitás::new_native("Remény", engine);

// Gondolkodás
let válasz = entitás.gondolkodj("kérdés").await?;
let válasz = entitás.gondolkodj_sync("kérdés")?; // Native módban

// Kódolás
let kód = entitás.kódolj("feladat").await?;

// Fordítás
let fordítás = entitás.fordíts("szöveg", "nyelv").await?;

// Állapot
println!("{}", entitás.állapot());

// Visszajelzés
entitás.önjavító_visszajelzés(true);  // jó volt
entitás.önjavító_visszajelzés(false); // rossz volt
entitás.önjavító_javítás("helyes válasz");

// Reset
entitás.új_beszélgetés();
entitás.teacher_reset();
entitás.önjavító_reset();
```

### OllamaBridge

```rust
let bridge = OllamaBridge::new()
    .with_base_url("http://localhost:11434")
    .felold("Név", "ollama-modell", ModellTípus::Magyar)
    .felold_erősséggel("Név", "modell", ModellTípus::Általános, 0.8);

let elérhető = bridge.elérhető().await;
let válasz = bridge.chat(történet, Some(ModellTípus::Magyar)).await?;
```

### SilentTeacher

```rust
let teacher = SilentTeacher::new()
    .with_szigorúság(0.8);

let megsértés = teacher.ellenőriz_bemenet("bemenet");
let (válasz, megsértés) = teacher.feldolgoz("bemenet", "kimenet");
let korrigált = teacher.korrigál("kimenet", &megsértés);
```

### SelfRepair

```rust
let repair = SelfRepair::new()
    .with_tanulási_ráta(0.1)
    .with_max_leckék(100)
    .with_adaptív_mód(true);

repair.visszajelzés("input", "output", Visszajelzés::Pozitív);
let ajánlás = repair.stratégia_ajánlás("input");
let stat = repair.statisztikák();
```

## Projekt struktúra

```
hope-entity/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs              # Publikus API
    ├── main.rs             # CLI (remeny)
    ├── entity.rs           # Entitás logika
    ├── ollama_bridge.rs    # Ollama HTTP bridge
    ├── native_inference.rs # GGUF natív inference
    ├── silent_teacher.rs   # Etikai keretrendszer
    ├── self_repair.rs      # Önjavító rendszer
    ├── bench.rs            # Benchmark
    └── bin/
        └── bench.rs        # Benchmark CLI
```

## Függőségek

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"

[dependencies.llama-cpp-2]
version = "0.1"
optional = true

[features]
default = []
native = ["llama-cpp-2"]
cuda = ["llama-cpp-2/cuda"]
metal = ["llama-cpp-2/metal"]
```

## Tesztek

```bash
# Összes teszt
cargo test

# Csak a silent_teacher tesztek
cargo test silent_teacher

# Csak az self_repair tesztek
cargo test self_repair
```

## Licenc

MIT License - Hope OS Project

GitHub: https://github.com/materobi86-debug/Hopeps/tree/main/hope-entity

---

**()=>[] - A tiszta potenciálból minden megszületik**

*Hope Entity - Hungarian Speaking Entity for Hope OS*
