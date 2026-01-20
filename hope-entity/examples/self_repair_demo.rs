//! Önjavító rendszer demonstráció
//!
//! Futtatás: cargo run --example self_repair_demo
//!
//! Ez a példa bemutatja hogyan tanul az entitás a hibáiból.

use hope_entity::{HibaTípus, SelfRepair, Visszajelzés};

fn main() {
    println!("🔧 Hope Entity - Önjavító Rendszer Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut repair = SelfRepair::new()
        .with_tanulási_ráta(0.2)
        .with_adaptív_mód(true);

    // 1. Kezdeti állapot
    println!("📊 Kezdeti állapot:");
    println!("{}\n", repair.állapot());

    // 2. Szimuláljunk néhány hibát
    println!("❌ Hibák szimulálása...\n");

    // Túl rövid válasz hiba
    repair.visszajelzés(
        "Mi a kvantumfizika és hogyan működik?",
        "Nem tudom.",
        Visszajelzés::Negatív(HibaTípus::TúlRövidVálasz)
    );
    println!("   • Túl rövid válasz regisztrálva");

    // Nyelvi hiba (magyar kérdésre angol válasz)
    repair.visszajelzés(
        "Szia, hogy vagy?",
        "I'm fine, thank you!",
        Visszajelzés::Negatív(HibaTípus::NyelviHiba)
    );
    println!("   • Nyelvi hiba regisztrálva");

    // Javítás megadása
    repair.visszajelzés(
        "Mi az a rekurzió?",
        "Valami programozás.",
        Visszajelzés::Javítás("A rekurzió egy programozási technika, ahol egy függvény önmagát hívja meg.".into())
    );
    println!("   • Javítás tanulva\n");

    // 3. Állapot a hibák után
    println!("📊 Állapot hibák után:");
    println!("{}\n", repair.állapot());

    // 4. Pozitív visszajelzések
    println!("✅ Pozitív visszajelzések...\n");

    for _ in 0..3 {
        repair.visszajelzés(
            "Valami kérdés",
            "Jó válasz",
            Visszajelzés::Pozitív
        );
    }
    println!("   • 3 sikeres interakció\n");

    // 5. Végső állapot
    println!("📊 Végső állapot:");
    println!("{}\n", repair.állapot());

    // 6. Tanult válasz lekérése
    println!("🎓 Tanult válasz teszt:");
    if let Some(válasz) = repair.helyes_válasz("Mi az a rekurzió?") {
        println!("   Kérdés: Mi az a rekurzió?");
        println!("   Tanult válasz: {}\n", válasz);
    }

    // 7. Stratégia ajánlás
    println!("🎯 Stratégia ajánlás teszt:");
    let ajánlás = repair.stratégia_ajánlás("Szia, hogy vagy?");
    println!("   Kérdés: Szia, hogy vagy?");
    if let Some(nyelv) = ajánlás.erőltetett_nyelv {
        println!("   Ajánlott nyelv: {}", nyelv);
    }
    if ajánlás.óvatos_mód {
        println!("   ⚠️ Óvatos mód aktív (alacsony önbizalom)");
    }
    println!();

    // 8. Statisztikák
    let stat = repair.statisztikák();
    println!("📈 Részletes statisztikák:");
    println!("   Tanult leckék: {}", stat.tanult_leckék);
    println!("   Összes hiba: {}", stat.összes_hiba);
    println!("   Sikeres javítások: {}", stat.sikeres_javítások);
    println!("   Javítási arány: {:.1}%", stat.javítási_arány * 100.0);
    println!("   Önbizalom: {:.1}%", stat.önbizalom * 100.0);
    if let Some(ref hiba) = stat.leggyakoribb_hiba {
        println!("   Leggyakoribb hiba: {:?}", hiba);
    }
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Az entitás tanul és fejlődik minden interakcióval!");
}
