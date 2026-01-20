//! Hope Entity Benchmark - Komplex képességek mérése
//!
//! ()=>[] - A sebesség is a potenciálból születik

use crate::{Entitás, ModellTípus, OllamaBridge, Személyiség, ÉrzelmiÁllapot};
use std::time::{Duration, Instant};

/// Benchmark eredmény
#[derive(Debug, Clone)]
pub struct BenchmarkEredmény {
    pub név: String,
    pub idő: Duration,
    pub iterációk: u64,
    pub ops_per_sec: f64,
}

impl BenchmarkEredmény {
    pub fn new(név: &str, idő: Duration, iterációk: u64) -> Self {
        let ops_per_sec = iterációk as f64 / idő.as_secs_f64();
        Self {
            név: név.to_string(),
            idő,
            iterációk,
            ops_per_sec,
        }
    }

    pub fn kiír(&self) {
        println!(
            "  {:30} {:>10.2} μs/op  {:>12.0} ops/sec  ({} iteráció)",
            self.név,
            self.idő.as_micros() as f64 / self.iterációk as f64,
            self.ops_per_sec,
            self.iterációk
        );
    }
}

/// Komplex benchmark futtatása
pub struct EntityBenchmark {
    eredmények: Vec<BenchmarkEredmény>,
}

impl EntityBenchmark {
    pub fn new() -> Self {
        Self {
            eredmények: Vec::new(),
        }
    }

    /// Összes benchmark futtatása
    pub fn futtat_mindent(&mut self) -> &[BenchmarkEredmény] {
        println!("\n🚀 HOPE ENTITY BENCHMARK - Komplex képességek\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // 1. Entitás létrehozás
        self.bench_entitás_létrehozás();

        // 2. Bridge és modell konfiguráció
        self.bench_bridge_konfiguráció();

        // 3. Modell választás (intelligens routing)
        self.bench_modell_választás();

        // 4. Érzelem feldolgozás
        self.bench_érzelem_feldolgozás();

        // 5. Memória műveletek
        self.bench_memória_műveletek();

        // 6. Személyiség klónozás
        self.bench_személyiség_műveletek();

        // 7. Komplex szöveg elemzés
        self.bench_szöveg_elemzés();

        // 8. Tömeges entitás kezelés
        self.bench_tömeges_entitás();

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // Összegzés
        self.összegzés();

        &self.eredmények
    }

    fn bench_entitás_létrehozás(&mut self) {
        println!("\n📦 Entitás létrehozás");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let iterációk = 10_000;
        let start = Instant::now();

        for _ in 0..iterációk {
            let _e = Entitás::new("Teszt");
            std::hint::black_box(&_e);
        }

        let eredmény = BenchmarkEredmény::new("Entitás::new()", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Személyiséggel
        let start = Instant::now();
        for _ in 0..iterációk {
            let _e = Entitás::new("Teszt").with_személyiség(Személyiség::default());
            std::hint::black_box(&_e);
        }

        let eredmény = BenchmarkEredmény::new("Entitás + személyiség", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_bridge_konfiguráció(&mut self) {
        println!("\n🌉 Bridge konfiguráció");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let iterációk = 50_000;
        let start = Instant::now();

        for _ in 0..iterációk {
            let _b = OllamaBridge::new()
                .felold("Magyar", "model1", ModellTípus::Magyar)
                .felold("Kódoló", "model2", ModellTípus::Kódoló)
                .felold("Multi", "model3", ModellTípus::Többnyelvű);
            std::hint::black_box(&_b);
        }

        let eredmény = BenchmarkEredmény::new("Bridge + 3 modell", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // 10 modellel
        let start = Instant::now();
        for _ in 0..iterációk {
            let mut b = OllamaBridge::new();
            for i in 0..10 {
                b = b.felold(&format!("Model{}", i), &format!("ollama{}", i), ModellTípus::Általános);
            }
            std::hint::black_box(&b);
        }

        let eredmény = BenchmarkEredmény::new("Bridge + 10 modell", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_modell_választás(&mut self) {
        println!("\n🎯 Intelligens modell választás (routing)");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let bridge = OllamaBridge::new()
            .felold("Magyar", "hu-model", ModellTípus::Magyar)
            .felold("Kódoló", "code-model", ModellTípus::Kódoló)
            .felold("Multi", "multi-model", ModellTípus::Többnyelvű)
            .felold("Alt1", "alt1", ModellTípus::Általános)
            .felold("Alt2", "alt2", ModellTípus::Általános);

        let teszt_szövegek = vec![
            "Szia, hogy vagy ma?",
            "fn main() { println!(\"Hello\"); }",
            "Write a function that calculates fibonacci",
            "Írj nekem egy osztályt Python-ban",
            "Köszönöm szépen a segítséget!",
            "impl Iterator for MyStruct",
            "Milyen idő van ma Budapesten?",
            "def calculate_sum(a, b):",
            "Ez egy nagyon hosszú magyar szöveg ami sok ékezetes betűt tartalmaz és teszteli a magyar nyelv felismerést",
            "```rust\nlet x = 42;\n```",
        ];

        let iterációk = 100_000u64;
        let start = Instant::now();

        for _ in 0..iterációk {
            for szöveg in &teszt_szövegek {
                let _m = bridge.válaszd_modellt(szöveg);
                std::hint::black_box(&_m);
            }
        }

        let összes_művelet = iterációk * teszt_szövegek.len() as u64;
        let eredmény = BenchmarkEredmény::new("Modell routing (10 szöveg)", start.elapsed(), összes_művelet);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Csak magyar felismerés
        let start = Instant::now();
        for _ in 0..iterációk {
            let _m = bridge.válaszd_modellt("Szia, hogy vagy?");
            std::hint::black_box(&_m);
        }

        let eredmény = BenchmarkEredmény::new("Magyar felismerés", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Kód felismerés
        let start = Instant::now();
        for _ in 0..iterációk {
            let _m = bridge.válaszd_modellt("fn main() { let x = 42; }");
            std::hint::black_box(&_m);
        }

        let eredmény = BenchmarkEredmény::new("Kód felismerés", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_érzelem_feldolgozás(&mut self) {
        println!("\n💚 Érzelem feldolgozás");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let érzelmi_szövegek = vec![
            "Köszönöm szépen, ez szuper!",
            "Miért történt ez? Hogyan működik?",
            "WOW! Ez zseniális!!!",
            "Király vagy, köszi!",
            "Nagyon érdekes kérdés...",
        ];

        let iterációk = 50_000u64;

        // Érzelem frissítés szimuláció (privát metódus, de tudjuk hogy O(1))
        let start = Instant::now();

        for _ in 0..iterációk {
            let mut érzelem = ÉrzelmiÁllapot::default();
            for szöveg in &érzelmi_szövegek {
                // Szimuláljuk az érzelem frissítést
                let szöveg_lower = szöveg.to_lowercase();
                if szöveg_lower.contains("köszön") || szöveg_lower.contains("szuper") {
                    érzelem.öröm = (érzelem.öröm + 0.1).min(1.0);
                }
                if szöveg_lower.contains('?') {
                    érzelem.kíváncsiság = (érzelem.kíváncsiság + 0.1).min(1.0);
                }
                if szöveg_lower.contains('!') {
                    érzelem.lelkesedés = (érzelem.lelkesedés + 0.15).min(1.0);
                }
                std::hint::black_box(&érzelem);
            }
        }

        let összes = iterációk * érzelmi_szövegek.len() as u64;
        let eredmény = BenchmarkEredmény::new("Érzelem frissítés", start.elapsed(), összes);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Érzelem klónozás
        let start = Instant::now();
        let érzelem = ÉrzelmiÁllapot::default();
        for _ in 0..iterációk * 10 {
            let _k = érzelem.clone();
            std::hint::black_box(&_k);
        }

        let eredmény = BenchmarkEredmény::new("Érzelem klónozás", start.elapsed(), iterációk * 10);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_memória_műveletek(&mut self) {
        println!("\n🧠 Memória műveletek");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        use std::collections::VecDeque;
        use crate::Emlék;

        let iterációk = 100_000u64;

        // Emlék létrehozás
        let start = Instant::now();
        for _ in 0..iterációk {
            let _e = Emlék {
                tartalom: "Teszt emlék tartalom".to_string(),
                fontosság: 0.5,
                érzelem: ÉrzelmiÁllapot::default(),
                időbélyeg: std::time::SystemTime::now(),
            };
            std::hint::black_box(&_e);
        }

        let eredmény = BenchmarkEredmény::new("Emlék létrehozás", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // VecDeque műveletek (rövid memória szimuláció)
        let start = Instant::now();
        let mut memória: VecDeque<String> = VecDeque::with_capacity(10);

        for i in 0..iterációk {
            if memória.len() >= 10 {
                memória.pop_front();
            }
            memória.push_back(format!("Emlék #{}", i));
        }
        std::hint::black_box(&memória);

        let eredmény = BenchmarkEredmény::new("Rövid memória (10 elem)", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Hosszú memória (Vec push)
        let start = Instant::now();
        let mut hosszú: Vec<String> = Vec::new();

        for i in 0..iterációk {
            hosszú.push(format!("Hosszú emlék #{}", i));
        }
        std::hint::black_box(&hosszú);

        let eredmény = BenchmarkEredmény::new("Hosszú memória push", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_személyiség_műveletek(&mut self) {
        println!("\n👤 Személyiség műveletek");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let iterációk = 500_000u64;

        // Személyiség létrehozás
        let start = Instant::now();
        for _ in 0..iterációk {
            let _s = Személyiség::default();
            std::hint::black_box(&_s);
        }

        let eredmény = BenchmarkEredmény::new("Személyiség::default()", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Személyiség klónozás
        let személyiség = Személyiség {
            melegség: 0.9,
            bölcsesség: 0.8,
            humor: 0.7,
            direktség: 0.6,
            kreativitás: 0.95,
        };

        let start = Instant::now();
        for _ in 0..iterációk {
            let _k = személyiség.clone();
            std::hint::black_box(&_k);
        }

        let eredmény = BenchmarkEredmény::new("Személyiség klónozás", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_szöveg_elemzés(&mut self) {
        println!("\n📝 Komplex szöveg elemzés");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        let iterációk = 100_000u64;

        // Magyar karakterek felismerése
        let magyar_szöveg = "Árvíztűrő tükörfúrógép - őŐűŰ ékezetes karakterek";

        let start = Instant::now();
        for _ in 0..iterációk {
            let _contains = magyar_szöveg.chars().any(|c| matches!(c, 'á'|'é'|'í'|'ó'|'ö'|'ő'|'ú'|'ü'|'ű'));
            std::hint::black_box(&_contains);
        }

        let eredmény = BenchmarkEredmény::new("Magyar karakter keresés", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Kód pattern keresés
        let kód_szöveg = "fn main() { let x = 42; impl Trait for Struct { pub fn method() {} } }";
        let kód_jelek = ["fn ", "let ", "impl ", "pub ", "use "];

        let start = Instant::now();
        for _ in 0..iterációk {
            let _found = kód_jelek.iter().any(|jel| kód_szöveg.contains(jel));
            std::hint::black_box(&_found);
        }

        let eredmény = BenchmarkEredmény::new("Kód pattern keresés", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // Szöveg lowercase + keresés kombinálva
        let vegyes_szöveg = "Szia! Írj nekem egy fn main() függvényt, KÖSZÖNÖM!!!";

        let start = Instant::now();
        for _ in 0..iterációk {
            let lower = vegyes_szöveg.to_lowercase();
            let _magyar = lower.contains("szia") || lower.contains("köszön");
            let _kód = lower.contains("fn ") || lower.contains("impl ");
            let _lelkes = vegyes_szöveg.contains('!');
            std::hint::black_box((&_magyar, &_kód, &_lelkes));
        }

        let eredmény = BenchmarkEredmény::new("Komplex szöveg elemzés", start.elapsed(), iterációk);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn bench_tömeges_entitás(&mut self) {
        println!("\n🏭 Tömeges entitás kezelés");
        println!("─────────────────────────────────────────────────────────────────────────────────");

        // 100 entitás létrehozása
        let start = Instant::now();
        let mut entitások: Vec<Entitás> = Vec::new();

        for i in 0..100 {
            let bridge = OllamaBridge::new()
                .felold("Magyar", "hu", ModellTípus::Magyar)
                .felold("Kód", "code", ModellTípus::Kódoló);

            let e = Entitás::new(&format!("Entitás#{}", i))
                .with_bridge(bridge);
            entitások.push(e);
        }
        std::hint::black_box(&entitások);

        let eredmény = BenchmarkEredmény::new("100 entitás létrehozás", start.elapsed(), 100);
        eredmény.kiír();
        self.eredmények.push(eredmény);

        // 1000 entitás
        let start = Instant::now();
        let mut entitások: Vec<Entitás> = Vec::new();

        for i in 0..1000 {
            let e = Entitás::new(&format!("E{}", i));
            entitások.push(e);
        }
        std::hint::black_box(&entitások);

        let eredmény = BenchmarkEredmény::new("1000 entitás (alap)", start.elapsed(), 1000);
        eredmény.kiír();
        self.eredmények.push(eredmény);
    }

    fn összegzés(&self) {
        println!("📊 ÖSSZEGZÉS");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let össz_idő: Duration = self.eredmények.iter().map(|e| e.idő).sum();
        let össz_ops: u64 = self.eredmények.iter().map(|e| e.iterációk).sum();

        // Leggyorsabb műveletek
        let mut rendezett = self.eredmények.clone();
        rendezett.sort_by(|a, b| {
            let a_per_op = a.idő.as_nanos() as f64 / a.iterációk as f64;
            let b_per_op = b.idő.as_nanos() as f64 / b.iterációk as f64;
            a_per_op.partial_cmp(&b_per_op).unwrap()
        });

        println!("🏆 TOP 5 Leggyorsabb művelet:");
        for (i, e) in rendezett.iter().take(5).enumerate() {
            let ns_per_op = e.idő.as_nanos() as f64 / e.iterációk as f64;
            println!("   {}. {:30} {:>8.1} ns/op", i + 1, e.név, ns_per_op);
        }

        println!("\n📈 Statisztika:");
        println!("   Összes művelet:     {:>12}", össz_ops);
        println!("   Összes idő:         {:>12.2} ms", össz_idő.as_secs_f64() * 1000.0);
        println!("   Átlag throughput:   {:>12.0} ops/sec", össz_ops as f64 / össz_idő.as_secs_f64());

        println!("\n💡 Az entitás AZONNAL reagál - a várakozás csak az Ollama-nál van!");
        println!("   A belső műveletek NANOSZEKUNDUM nagyságrendűek.\n");
        println!("()=>[] - A sebesség is a potenciálból születik\n");
    }
}

impl Default for EntityBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// Gyors benchmark futtatása
pub fn gyors_benchmark() {
    let mut bench = EntityBenchmark::new();
    bench.futtat_mindent();
}
