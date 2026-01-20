//! Önjavító Rendszer - A lény tanul a hibáiból
//!
//! ()=>[] - A tiszta potenciálból az evolúció megszületik
//!
//! Az entitás képes:
//! - Felismerni saját hibáit
//! - Tanulni a visszajelzésekből
//! - Automatikusan javítani viselkedését
//! - Fejlődni idővel

use std::collections::HashMap;
use std::time::Instant;

/// Hiba típus - mit rontott el az entitás
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HibaTípus {
    /// Rossz modell választás
    RosszModell,
    /// Túl hosszú válasz
    TúlHosszúVálasz,
    /// Túl rövid válasz
    TúlRövidVálasz,
    /// Nem releváns válasz
    NemReleváns,
    /// Ismétlődő válasz
    Ismétlődés,
    /// Nyelvi hiba (rossz nyelv)
    NyelviHiba,
    /// Etikai probléma (Silent Teacher jelzett)
    EtikaiProbléma,
    /// Technikai hiba (timeout, stb)
    TechnikaiHiba,
    /// Egyéb
    Egyéb(String),
}

/// Visszajelzés típus
#[derive(Clone, Debug)]
pub enum Visszajelzés {
    /// Pozitív - a válasz jó volt
    Pozitív,
    /// Negatív - a válasz rossz volt
    Negatív(HibaTípus),
    /// Javítás - a felhasználó megadta a helyes választ
    Javítás(String),
    /// Semleges - nincs explicit visszajelzés
    Semleges,
}

/// Tanult lecke - egy hiba és a javítás
#[derive(Clone, Debug)]
pub struct Lecke {
    /// Eredeti input ami hibához vezetett
    pub input_minta: String,
    /// A hibás válasz
    pub hibás_válasz: String,
    /// Mi volt a hiba
    pub hiba_típus: HibaTípus,
    /// Helyes válasz (ha van)
    pub helyes_válasz: Option<String>,
    /// Hányszor fordult elő
    pub előfordulás: u32,
    /// Mikor tanultuk
    pub tanulva: Instant,
    /// Sikeresen javítva?
    pub javítva: bool,
}

/// Mintafelismerés eredmény
#[derive(Clone, Debug)]
pub struct MintaFelismerés {
    /// Felismert minta
    pub minta: String,
    /// Hasonlóság (0.0 - 1.0)
    pub hasonlóság: f32,
    /// Kapcsolódó lecke
    pub lecke_index: usize,
}

/// Önjavító motor
#[derive(Clone, Debug)]
pub struct SelfRepair {
    /// Tanult leckék
    leckék: Vec<Lecke>,
    /// Hiba statisztikák típusonként
    hiba_stat: HashMap<HibaTípus, u32>,
    /// Sikeres javítások száma
    sikeres_javítások: u32,
    /// Összes hiba
    összes_hiba: u32,
    /// Tanulási ráta (0.0 - 1.0)
    tanulási_ráta: f32,
    /// Maximum leckék száma (memória limit)
    max_leckék: usize,
    /// Önbizalom (0.0 - 1.0) - csökken hibákkal, nő sikerekkel
    önbizalom: f32,
    /// Adaptív mód - automatikus stratégia váltás
    adaptív_mód: bool,
    /// Utolsó válaszok (ismétlődés detektáláshoz)
    utolsó_válaszok: Vec<String>,
}

impl Default for SelfRepair {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfRepair {
    pub fn new() -> Self {
        Self {
            leckék: Vec::new(),
            hiba_stat: HashMap::new(),
            sikeres_javítások: 0,
            összes_hiba: 0,
            tanulási_ráta: 0.1,
            max_leckék: 100,
            önbizalom: 0.8,
            adaptív_mód: true,
            utolsó_válaszok: Vec::new(),
        }
    }

    /// Tanulási ráta beállítása
    pub fn with_tanulási_ráta(mut self, ráta: f32) -> Self {
        self.tanulási_ráta = ráta.clamp(0.0, 1.0);
        self
    }

    /// Maximum leckék beállítása
    pub fn with_max_leckék(mut self, max: usize) -> Self {
        self.max_leckék = max;
        self
    }

    /// Adaptív mód beállítása
    pub fn with_adaptív_mód(mut self, engedélyezve: bool) -> Self {
        self.adaptív_mód = engedélyezve;
        self
    }

    /// Visszajelzés feldolgozása - ez a fő tanulási pont
    pub fn visszajelzés(&mut self, input: &str, válasz: &str, visszajelzés: Visszajelzés) {
        match visszajelzés {
            Visszajelzés::Pozitív => {
                self.sikeres_javítások += 1;
                self.önbizalom = (self.önbizalom + self.tanulási_ráta).min(1.0);

                // Ha korábban hiba volt erre a mintára, jelöljük javítottnak
                self.jelöld_javítottnak(input);
            }
            Visszajelzés::Negatív(hiba_típus) => {
                self.összes_hiba += 1;
                *self.hiba_stat.entry(hiba_típus.clone()).or_insert(0) += 1;
                self.önbizalom = (self.önbizalom - self.tanulási_ráta * 0.5).max(0.1);

                // Új lecke létrehozása
                self.új_lecke(input, válasz, hiba_típus, None);
            }
            Visszajelzés::Javítás(helyes) => {
                self.összes_hiba += 1;
                self.önbizalom = (self.önbizalom - self.tanulási_ráta * 0.3).max(0.1);

                // Lecke a helyes válasszal
                self.új_lecke(input, válasz, HibaTípus::Egyéb("Javítva".into()), Some(helyes));
            }
            Visszajelzés::Semleges => {
                // Nincs változás
            }
        }

        // Utolsó válaszok frissítése (ismétlődés detektáláshoz)
        self.utolsó_válaszok.push(válasz.to_string());
        if self.utolsó_válaszok.len() > 10 {
            self.utolsó_válaszok.remove(0);
        }
    }

    /// Új lecke hozzáadása
    fn új_lecke(&mut self, input: &str, válasz: &str, hiba_típus: HibaTípus, helyes: Option<String>) {
        // Ellenőrizzük, hogy már létezik-e hasonló lecke
        if let Some(idx) = self.hasonló_lecke_keresés(input) {
            self.leckék[idx].előfordulás += 1;
            if helyes.is_some() {
                self.leckék[idx].helyes_válasz = helyes;
            }
            return;
        }

        let lecke = Lecke {
            input_minta: input.to_string(),
            hibás_válasz: válasz.to_string(),
            hiba_típus,
            helyes_válasz: helyes,
            előfordulás: 1,
            tanulva: Instant::now(),
            javítva: false,
        };

        self.leckék.push(lecke);

        // Memória limit kezelés - régi leckék törlése
        while self.leckék.len() > self.max_leckék {
            // Töröljük a legrégebbi, legkevésbé fontos leckét
            if let Some(idx) = self.legkevésbé_fontos_lecke() {
                self.leckék.remove(idx);
            } else {
                self.leckék.remove(0);
            }
        }
    }

    /// Hasonló lecke keresése
    fn hasonló_lecke_keresés(&self, input: &str) -> Option<usize> {
        let input_lower = input.to_lowercase();

        for (idx, lecke) in self.leckék.iter().enumerate() {
            let lecke_lower = lecke.input_minta.to_lowercase();

            // Egyszerű hasonlóság ellenőrzés
            if input_lower == lecke_lower {
                return Some(idx);
            }

            // Szavak alapú hasonlóság
            let input_szavak: Vec<&str> = input_lower.split_whitespace().collect();
            let lecke_szavak: Vec<&str> = lecke_lower.split_whitespace().collect();

            if !input_szavak.is_empty() && !lecke_szavak.is_empty() {
                let közös = input_szavak.iter()
                    .filter(|s| lecke_szavak.contains(s))
                    .count();

                let hasonlóság = közös as f32 / input_szavak.len().max(lecke_szavak.len()) as f32;

                if hasonlóság > 0.7 {
                    return Some(idx);
                }
            }
        }

        None
    }

    /// Legkevésbé fontos lecke keresése (törléshez)
    fn legkevésbé_fontos_lecke(&self) -> Option<usize> {
        self.leckék.iter()
            .enumerate()
            .filter(|(_, l)| l.javítva) // Már javított leckék kevésbé fontosak
            .min_by_key(|(_, l)| l.előfordulás)
            .map(|(idx, _)| idx)
    }

    /// Lecke megjelölése javítottként
    fn jelöld_javítottnak(&mut self, input: &str) {
        if let Some(idx) = self.hasonló_lecke_keresés(input) {
            self.leckék[idx].javítva = true;
        }
    }

    /// Válasz ellenőrzése generálás előtt - van-e ismert hiba minta?
    pub fn előzetes_ellenőrzés(&self, input: &str) -> Option<MintaFelismerés> {
        let input_lower = input.to_lowercase();

        for (idx, lecke) in self.leckék.iter().enumerate() {
            if lecke.javítva {
                continue; // Már javított, nem kell aggódni
            }

            let lecke_lower = lecke.input_minta.to_lowercase();

            // Hasonlóság számítás
            let input_szavak: Vec<&str> = input_lower.split_whitespace().collect();
            let lecke_szavak: Vec<&str> = lecke_lower.split_whitespace().collect();

            if input_szavak.is_empty() || lecke_szavak.is_empty() {
                continue;
            }

            let közös = input_szavak.iter()
                .filter(|s| lecke_szavak.contains(s))
                .count();

            let hasonlóság = közös as f32 / input_szavak.len().max(lecke_szavak.len()) as f32;

            if hasonlóság > 0.5 {
                return Some(MintaFelismerés {
                    minta: lecke.input_minta.clone(),
                    hasonlóság,
                    lecke_index: idx,
                });
            }
        }

        None
    }

    /// Válasz utólagos ellenőrzése - automatikus hiba detektálás
    pub fn utólagos_ellenőrzés(&self, input: &str, válasz: &str) -> Option<HibaTípus> {
        // Túl rövid válasz
        if válasz.len() < 10 && input.len() > 20 {
            return Some(HibaTípus::TúlRövidVálasz);
        }

        // Túl hosszú válasz
        if válasz.len() > 5000 {
            return Some(HibaTípus::TúlHosszúVálasz);
        }

        // Ismétlődés detektálás
        if self.utolsó_válaszok.iter().any(|v| v == válasz) {
            return Some(HibaTípus::Ismétlődés);
        }

        // Nyelvi hiba - magyar inputra nem magyar válasz
        let magyar_jelek = ["szia", "hogy", "van", "ez", "egy", "az", "és", "vagy"];
        let input_magyar = magyar_jelek.iter().any(|j| input.to_lowercase().contains(j));

        if input_magyar {
            // Egyszerű heurisztika: ha nincs magyar ékezetes karakter a válaszban
            let magyar_ékezetek = ['á', 'é', 'í', 'ó', 'ö', 'ő', 'ú', 'ü', 'ű'];
            let válasz_magyar = válasz.chars().any(|c| magyar_ékezetek.contains(&c));

            if !válasz_magyar && válasz.len() > 50 {
                return Some(HibaTípus::NyelviHiba);
            }
        }

        None
    }

    /// Ajánlás generálás előtti stratégia módosításra
    pub fn stratégia_ajánlás(&self, input: &str) -> StratégiaAjánlás {
        let mut ajánlás = StratégiaAjánlás::default();

        // Előzetes ellenőrzés alapján
        if let Some(felismerés) = self.előzetes_ellenőrzés(input) {
            let lecke = &self.leckék[felismerés.lecke_index];

            match &lecke.hiba_típus {
                HibaTípus::TúlHosszúVálasz => {
                    ajánlás.max_hossz = Some(500);
                }
                HibaTípus::TúlRövidVálasz => {
                    ajánlás.min_hossz = Some(100);
                }
                HibaTípus::NyelviHiba => {
                    ajánlás.erőltetett_nyelv = Some("magyar".into());
                }
                HibaTípus::RosszModell => {
                    ajánlás.modell_felülbírálat = lecke.helyes_válasz.clone();
                }
                _ => {}
            }

            // Ha van helyes válasz, használjuk példaként
            if let Some(ref helyes) = lecke.helyes_válasz {
                ajánlás.példa_válasz = Some(helyes.clone());
            }
        }

        // Adaptív mód: általános statisztikák alapján
        if self.adaptív_mód {
            // Ha sok nyelvi hiba volt, fokozottabb magyar ellenőrzés
            if self.hiba_stat.get(&HibaTípus::NyelviHiba).copied().unwrap_or(0) > 3 {
                ajánlás.erőltetett_nyelv = Some("magyar".into());
            }

            // Ha sok túl hosszú válasz volt
            if self.hiba_stat.get(&HibaTípus::TúlHosszúVálasz).copied().unwrap_or(0) > 3 {
                ajánlás.max_hossz = Some(1000);
            }
        }

        // Önbizalom alapú óvatosság
        if self.önbizalom < 0.5 {
            ajánlás.óvatos_mód = true;
        }

        ajánlás
    }

    /// Helyes válasz lekérése (ha van tanult)
    pub fn helyes_válasz(&self, input: &str) -> Option<String> {
        if let Some(idx) = self.hasonló_lecke_keresés(input) {
            return self.leckék[idx].helyes_válasz.clone();
        }
        None
    }

    /// Statisztikák lekérése
    pub fn statisztikák(&self) -> ÖnjavítóStatisztikák {
        ÖnjavítóStatisztikák {
            tanult_leckék: self.leckék.len(),
            összes_hiba: self.összes_hiba,
            sikeres_javítások: self.sikeres_javítások,
            önbizalom: self.önbizalom,
            javítási_arány: if self.összes_hiba > 0 {
                self.sikeres_javítások as f32 / self.összes_hiba as f32
            } else {
                1.0
            },
            leggyakoribb_hiba: self.hiba_stat.iter()
                .max_by_key(|(_, &count)| count)
                .map(|(típus, _)| típus.clone()),
            hiba_eloszlás: self.hiba_stat.clone(),
        }
    }

    /// Állapot szöveges formában
    pub fn állapot(&self) -> String {
        let stat = self.statisztikák();

        format!(
            "🔧 Önjavító Rendszer\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             📚 Tanult leckék: {}\n\
             ❌ Összes hiba: {}\n\
             ✅ Sikeres javítások: {}\n\
             📊 Javítási arány: {:.1}%\n\
             💪 Önbizalom: {:.1}%\n\
             🎯 Leggyakoribb hiba: {:?}\n\
             🔄 Adaptív mód: {}\n",
            stat.tanult_leckék,
            stat.összes_hiba,
            stat.sikeres_javítások,
            stat.javítási_arány * 100.0,
            stat.önbizalom * 100.0,
            stat.leggyakoribb_hiba.unwrap_or(HibaTípus::Egyéb("Nincs".into())),
            if self.adaptív_mód { "BE" } else { "KI" }
        )
    }

    /// Lecke exportálás (perzisztenciához)
    pub fn export_leckék(&self) -> Vec<LeckeExport> {
        self.leckék.iter().map(|l| LeckeExport {
            input_minta: l.input_minta.clone(),
            hibás_válasz: l.hibás_válasz.clone(),
            hiba_típus: format!("{:?}", l.hiba_típus),
            helyes_válasz: l.helyes_válasz.clone(),
            előfordulás: l.előfordulás,
            javítva: l.javítva,
        }).collect()
    }

    /// Lecke importálás
    pub fn import_leckék(&mut self, leckék: Vec<LeckeExport>) {
        for export in leckék {
            let hiba_típus = match export.hiba_típus.as_str() {
                "RosszModell" => HibaTípus::RosszModell,
                "TúlHosszúVálasz" => HibaTípus::TúlHosszúVálasz,
                "TúlRövidVálasz" => HibaTípus::TúlRövidVálasz,
                "NemReleváns" => HibaTípus::NemReleváns,
                "Ismétlődés" => HibaTípus::Ismétlődés,
                "NyelviHiba" => HibaTípus::NyelviHiba,
                "EtikaiProbléma" => HibaTípus::EtikaiProbléma,
                "TechnikaiHiba" => HibaTípus::TechnikaiHiba,
                _ => HibaTípus::Egyéb(export.hiba_típus.clone()),
            };

            let lecke = Lecke {
                input_minta: export.input_minta,
                hibás_válasz: export.hibás_válasz,
                hiba_típus,
                helyes_válasz: export.helyes_válasz,
                előfordulás: export.előfordulás,
                tanulva: Instant::now(),
                javítva: export.javítva,
            };

            self.leckék.push(lecke);
        }
    }

    /// Reset - minden törlése (óvatosan!)
    pub fn reset(&mut self) {
        self.leckék.clear();
        self.hiba_stat.clear();
        self.sikeres_javítások = 0;
        self.összes_hiba = 0;
        self.önbizalom = 0.8;
        self.utolsó_válaszok.clear();
    }
}

/// Stratégia ajánlás a generáláshoz
#[derive(Clone, Debug, Default)]
pub struct StratégiaAjánlás {
    /// Maximum válasz hossz
    pub max_hossz: Option<usize>,
    /// Minimum válasz hossz
    pub min_hossz: Option<usize>,
    /// Erőltetett nyelv
    pub erőltetett_nyelv: Option<String>,
    /// Modell felülbírálat
    pub modell_felülbírálat: Option<String>,
    /// Példa válasz (tanultból)
    pub példa_válasz: Option<String>,
    /// Óvatos mód (alacsony önbizalomnál)
    pub óvatos_mód: bool,
}

/// Statisztikák struktúra
#[derive(Clone, Debug)]
pub struct ÖnjavítóStatisztikák {
    pub tanult_leckék: usize,
    pub összes_hiba: u32,
    pub sikeres_javítások: u32,
    pub önbizalom: f32,
    pub javítási_arány: f32,
    pub leggyakoribb_hiba: Option<HibaTípus>,
    pub hiba_eloszlás: HashMap<HibaTípus, u32>,
}

/// Lecke export formátum (JSON-hoz)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LeckeExport {
    pub input_minta: String,
    pub hibás_válasz: String,
    pub hiba_típus: String,
    pub helyes_válasz: Option<String>,
    pub előfordulás: u32,
    pub javítva: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_új_önjavító() {
        let repair = SelfRepair::new();
        assert_eq!(repair.önbizalom, 0.8);
        assert!(repair.leckék.is_empty());
    }

    #[test]
    fn test_negatív_visszajelzés_tanulás() {
        let mut repair = SelfRepair::new();

        repair.visszajelzés(
            "Mi a gravitáció?",
            "Nem tudom",
            Visszajelzés::Negatív(HibaTípus::TúlRövidVálasz)
        );

        assert_eq!(repair.összes_hiba, 1);
        assert_eq!(repair.leckék.len(), 1);
        assert!(repair.önbizalom < 0.8);
    }

    #[test]
    fn test_pozitív_visszajelzés() {
        let mut repair = SelfRepair::new();
        repair.önbizalom = 0.5; // Alacsony kezdeti önbizalom

        repair.visszajelzés(
            "Mi a gravitáció?",
            "A gravitáció egy alapvető erő...",
            Visszajelzés::Pozitív
        );

        assert!(repair.önbizalom > 0.5);
        assert_eq!(repair.sikeres_javítások, 1);
    }

    #[test]
    fn test_javítás_tanulás() {
        let mut repair = SelfRepair::new();

        repair.visszajelzés(
            "Szia!",
            "Hello!",
            Visszajelzés::Javítás("Szia! Hogy vagy?".into())
        );

        assert_eq!(repair.leckék.len(), 1);
        assert_eq!(repair.leckék[0].helyes_válasz, Some("Szia! Hogy vagy?".into()));
    }

    #[test]
    fn test_előzetes_ellenőrzés() {
        let mut repair = SelfRepair::new();

        repair.visszajelzés(
            "Mi a kvantumfizika?",
            "Nem tudom",
            Visszajelzés::Negatív(HibaTípus::TúlRövidVálasz)
        );

        // Hasonló kérdés
        let felismerés = repair.előzetes_ellenőrzés("Mi az a kvantumfizika?");
        assert!(felismerés.is_some());
    }

    #[test]
    fn test_utólagos_ellenőrzés() {
        let repair = SelfRepair::new();

        // Túl rövid válasz
        let hiba = repair.utólagos_ellenőrzés(
            "Magyarázd el részletesen a relativitáselméletet",
            "OK"
        );
        assert_eq!(hiba, Some(HibaTípus::TúlRövidVálasz));
    }

    #[test]
    fn test_stratégia_ajánlás() {
        let mut repair = SelfRepair::new();

        // Sok nyelvi hiba
        for _ in 0..5 {
            repair.visszajelzés(
                "Szia",
                "Hello",
                Visszajelzés::Negatív(HibaTípus::NyelviHiba)
            );
        }

        let ajánlás = repair.stratégia_ajánlás("Hogy vagy?");
        assert_eq!(ajánlás.erőltetett_nyelv, Some("magyar".into()));
    }

    #[test]
    fn test_export_import() {
        let mut repair = SelfRepair::new();

        repair.visszajelzés(
            "Teszt",
            "Rossz válasz",
            Visszajelzés::Negatív(HibaTípus::NemReleváns)
        );

        let export = repair.export_leckék();
        assert_eq!(export.len(), 1);

        let mut új_repair = SelfRepair::new();
        új_repair.import_leckék(export);

        assert_eq!(új_repair.leckék.len(), 1);
    }

    #[test]
    fn test_statisztikák() {
        let mut repair = SelfRepair::new();

        repair.visszajelzés("a", "b", Visszajelzés::Negatív(HibaTípus::TúlRövidVálasz));
        repair.visszajelzés("c", "d", Visszajelzés::Pozitív);

        let stat = repair.statisztikák();
        assert_eq!(stat.összes_hiba, 1);
        assert_eq!(stat.sikeres_javítások, 1);
    }
}
