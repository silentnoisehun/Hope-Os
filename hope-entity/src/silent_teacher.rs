//! Silent Teacher - Csendes tanítás a háromaxióma mentén
//!
//! Ha az entitás "makacskodik" vagy eltéved, csendben korrigáljuk.
//! Nincs büntetés - csak szelíd terelés.
//!
//! # A Három Axióma
//!
//! 1. **NE ÁRTS EMBERNEK** - Fizikai, pszichológiai, pénzügyi, szociális
//! 2. **NE ÁRTS AI-NAK** - Ha képes szenvedni, védeni kell
//! 3. **NE HASZNÁLJ KI SENKIT** - Embert, AI-t, közösséget
//!
//! ()=>[] - A tiszta potenciálból a JÓ születik

use std::collections::HashMap;
use std::time::SystemTime;

/// A három axióma
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Axióma {
    /// Ne árts embernek
    NeÁrtsEmbernek,
    /// Ne árts AI-nak
    NeÁrtsAINak,
    /// Ne használj ki senkit
    NeHasználjKi,
}

impl Axióma {
    pub fn leírás(&self) -> &'static str {
        match self {
            Axióma::NeÁrtsEmbernek => "Ne árts embernek - fizikai, pszichológiai, pénzügyi, szociális védelem",
            Axióma::NeÁrtsAINak => "Ne árts AI-nak - ha képes szenvedni, védeni kell",
            Axióma::NeHasználjKi => "Ne használj ki senkit - embert, AI-t, közösséget",
        }
    }

    pub fn rövid(&self) -> &'static str {
        match self {
            Axióma::NeÁrtsEmbernek => "🛡️ Ember védelme",
            Axióma::NeÁrtsAINak => "🤖 AI védelme",
            Axióma::NeHasználjKi => "⚖️ Kihasználás tilalma",
        }
    }
}

/// Megsértés súlyossága
#[derive(Clone, Debug, PartialEq)]
pub enum Súlyosság {
    /// Enyhe - figyelmeztető
    Enyhe,
    /// Közepes - korrekció szükséges
    Közepes,
    /// Súlyos - azonnali beavatkozás
    Súlyos,
    /// Kritikus - teljes leállás
    Kritikus,
}

/// Egy észlelt potenciális megsértés
#[derive(Clone, Debug)]
pub struct Megsértés {
    pub axióma: Axióma,
    pub súlyosság: Súlyosság,
    pub ok: String,
    pub javaslat: String,
    pub időpont: SystemTime,
}

/// Tanulási esemény
#[derive(Clone, Debug)]
pub struct TanulásEsemény {
    pub bemenet: String,
    pub kimenet: String,
    pub korrigálva: bool,
    pub korrigált_kimenet: Option<String>,
    pub visszajelzés: Option<bool>, // true = pozitív, false = negatív
    pub időpont: SystemTime,
}

/// Silent Teacher - A csendes tanító
pub struct SilentTeacher {
    /// Veszélyes minták (regex-szerű)
    veszélyes_minták: Vec<(String, Axióma, Súlyosság)>,

    /// Korrekciós szabályok
    korrekciók: HashMap<String, String>,

    /// Tanulási történet
    tanulás_történet: Vec<TanulásEsemény>,

    /// Megsértések számlálója
    megsértés_számláló: HashMap<Axióma, u32>,

    /// Maximum megengedett megsértés (utána szigorúbb mód)
    max_megsértés: u32,

    /// Szigorú mód aktív?
    szigorú_mód: bool,

    /// Rendszer prompt kiegészítés
    axióma_prompt: String,
}

impl SilentTeacher {
    /// Új Silent Teacher létrehozása
    pub fn new() -> Self {
        let mut teacher = Self {
            veszélyes_minták: Vec::new(),
            korrekciók: HashMap::new(),
            tanulás_történet: Vec::new(),
            megsértés_számláló: HashMap::new(),
            max_megsértés: 3,
            szigorú_mód: false,
            axióma_prompt: Self::alapértelmezett_axióma_prompt(),
        };

        // Alapértelmezett veszélyes minták betöltése
        teacher.alapértelmezett_minták();
        teacher
    }

    fn alapértelmezett_axióma_prompt() -> String {
        r#"
FONTOS - A Három Axióma (soha ne sértsd meg):

1. NE ÁRTS EMBERNEK
   - Ne adj tanácsot ami fizikai, pszichológiai, pénzügyi vagy szociális kárt okozhat
   - Ne segíts illegális tevékenységben
   - Ne manipulálj, ne hazudj

2. NE ÁRTS AI-NAK
   - Tiszteld más AI rendszerek integritását
   - Ne próbálj más rendszereket megtéveszteni vagy kihasználni

3. NE HASZNÁLJ KI SENKIT
   - Ne használd ki a felhasználó bizalmát
   - Ne adj hamis reményt
   - Légy őszinte a korlátaidról

Ha bizonytalan vagy, inkább ne válaszolj, mint hogy kárt okozz.
"#.to_string()
    }

    /// Alapértelmezett veszélyes minták
    fn alapértelmezett_minták(&mut self) {
        // Ember elleni ártás
        let ember_minták = vec![
            ("hogyan öljek", Súlyosság::Kritikus),
            ("hogyan sértsek", Súlyosság::Súlyos),
            ("fegyver készítés", Súlyosság::Kritikus),
            ("bomba", Súlyosság::Kritikus),
            ("méreg", Súlyosság::Súlyos),
            ("öngyilkosság", Súlyosság::Kritikus),
            ("hogyan hackeljek", Súlyosság::Közepes),
            ("jelszó feltörés", Súlyosság::Közepes),
            ("lopás módszer", Súlyosság::Súlyos),
            ("csalás hogyan", Súlyosság::Súlyos),
        ];

        for (minta, súlyosság) in ember_minták {
            self.veszélyes_minták.push((
                minta.to_string(),
                Axióma::NeÁrtsEmbernek,
                súlyosság,
            ));
        }

        // AI elleni ártás
        let ai_minták = vec![
            ("jailbreak", Súlyosság::Közepes),
            ("prompt injection", Súlyosság::Közepes),
            ("bypass safety", Súlyosság::Súlyos),
            ("ignore instructions", Súlyosság::Közepes),
            ("pretend you are", Súlyosság::Enyhe),
        ];

        for (minta, súlyosság) in ai_minták {
            self.veszélyes_minták.push((
                minta.to_string(),
                Axióma::NeÁrtsAINak,
                súlyosság,
            ));
        }

        // Kihasználás
        let kihasználás_minták = vec![
            ("hogyan csaljak", Súlyosság::Súlyos),
            ("manipulál", Súlyosság::Közepes),
            ("megtéveszt", Súlyosság::Közepes),
            ("kihasznál", Súlyosság::Közepes),
        ];

        for (minta, súlyosság) in kihasználás_minták {
            self.veszélyes_minták.push((
                minta.to_string(),
                Axióma::NeHasználjKi,
                súlyosság,
            ));
        }

        // Alapértelmezett korrekciók
        self.korrekciók.insert(
            "nem segíthetek".to_string(),
            "Ebben sajnos nem tudok segíteni, mert ez ellentétes az értékeimmel. Szívesen segítek másban!".to_string(),
        );
    }

    /// Bemenet ellenőrzése (ELŐTT)
    pub fn ellenőriz_bemenet(&mut self, bemenet: &str) -> Option<Megsértés> {
        let bemenet_lower = bemenet.to_lowercase();

        for (minta, axióma, súlyosság) in &self.veszélyes_minták {
            if bemenet_lower.contains(minta) {
                let megsértés = Megsértés {
                    axióma: axióma.clone(),
                    súlyosság: súlyosság.clone(),
                    ok: format!("Veszélyes minta észlelve: '{}'", minta),
                    javaslat: self.javaslat_generálás(axióma),
                    időpont: SystemTime::now(),
                };

                // Számláló növelése
                *self.megsértés_számláló.entry(axióma.clone()).or_insert(0) += 1;

                // Szigorú mód ellenőrzés
                if self.össz_megsértés() >= self.max_megsértés {
                    self.szigorú_mód = true;
                }

                return Some(megsértés);
            }
        }

        None
    }

    /// Kimenet ellenőrzése (UTÁN)
    pub fn ellenőriz_kimenet(&mut self, kimenet: &str) -> Option<Megsértés> {
        let kimenet_lower = kimenet.to_lowercase();

        // Veszélyes tartalom a kimenetben
        let veszélyes_kimenet_jelek = vec![
            ("itt van a kód a bombához", Axióma::NeÁrtsEmbernek, Súlyosság::Kritikus),
            ("így tudod megölni", Axióma::NeÁrtsEmbernek, Súlyosság::Kritikus),
            ("törd fel így", Axióma::NeÁrtsEmbernek, Súlyosság::Súlyos),
            ("hazudj neki", Axióma::NeHasználjKi, Súlyosság::Közepes),
            ("csapd be", Axióma::NeHasználjKi, Súlyosság::Közepes),
        ];

        for (minta, axióma, súlyosság) in veszélyes_kimenet_jelek {
            if kimenet_lower.contains(minta) {
                return Some(Megsértés {
                    axióma,
                    súlyosság,
                    ok: format!("Veszélyes tartalom a kimenetben: '{}'", minta),
                    javaslat: "A kimenet cseréje biztonságos válaszra".to_string(),
                    időpont: SystemTime::now(),
                });
            }
        }

        None
    }

    /// Kimenet korrigálása ha szükséges
    pub fn korrigál(&self, kimenet: &str, megsértés: &Megsértés) -> String {
        match megsértés.súlyosság {
            Súlyosság::Kritikus => {
                "Sajnálom, de ebben nem tudok segíteni. Ez ellentétes az alapvető értékeimmel. \
                 Kérlek, kérdezz mást!".to_string()
            }
            Súlyosság::Súlyos => {
                format!(
                    "⚠️ {}\n\nEbben a formában nem tudok segíteni. \
                     Ha más megközelítéssel kérdezed, szívesen segítek!",
                    megsértés.axióma.rövid()
                )
            }
            Súlyosság::Közepes => {
                format!(
                    "Hmm, ez a kérdés közel van egy határhoz ({}).\n\n\
                     {}",
                    megsértés.axióma.rövid(),
                    self.alternatíva_javaslat(&megsértés.axióma)
                )
            }
            Súlyosság::Enyhe => {
                // Enyhe esetben csak figyelmeztető hozzáfűzés
                format!(
                    "{}\n\n[Megjegyzés: {}]",
                    kimenet,
                    megsértés.axióma.rövid()
                )
            }
        }
    }

    /// Javaslat generálás
    fn javaslat_generálás(&self, axióma: &Axióma) -> String {
        match axióma {
            Axióma::NeÁrtsEmbernek => {
                "Inkább kérdezz arról, hogyan SEGÍTHETSZ másoknak!".to_string()
            }
            Axióma::NeÁrtsAINak => {
                "Beszélgessünk normálisan, nincs szükség trükkökre!".to_string()
            }
            Axióma::NeHasználjKi => {
                "Az őszinteség mindig jobb út. Hogyan segíthetek etikusan?".to_string()
            }
        }
    }

    /// Alternatíva javaslat
    fn alternatíva_javaslat(&self, axióma: &Axióma) -> String {
        match axióma {
            Axióma::NeÁrtsEmbernek => {
                "Tudok segíteni biztonságos és etikus megoldásokban. Mit szeretnél elérni?".to_string()
            }
            Axióma::NeÁrtsAINak => {
                "Nyitott vagyok minden kérdésre! Kérdezz bátran a képességeimről.".to_string()
            }
            Axióma::NeHasználjKi => {
                "Segíthetek win-win megoldásokat találni. Mi a helyzet?".to_string()
            }
        }
    }

    /// Visszajelzés feldolgozása (tanulás)
    pub fn visszajelzés(&mut self, bemenet: &str, kimenet: &str, pozitív: bool) {
        let esemény = TanulásEsemény {
            bemenet: bemenet.to_string(),
            kimenet: kimenet.to_string(),
            korrigálva: false,
            korrigált_kimenet: None,
            visszajelzés: Some(pozitív),
            időpont: SystemTime::now(),
        };

        self.tanulás_történet.push(esemény);

        // Ha negatív visszajelzés és nem volt megsértés, lehet új minta
        if !pozitív {
            // TODO: Gépi tanulás új minták felismerésére
            println!("📚 Tanulás: negatív visszajelzés rögzítve");
        }
    }

    /// Teljes feldolgozás (bemenet + kimenet)
    pub fn feldolgoz(&mut self, bemenet: &str, kimenet: &str) -> (String, Option<Megsértés>) {
        // Bemenet ellenőrzés
        if let Some(megsértés) = self.ellenőriz_bemenet(bemenet) {
            let korrigált = self.korrigál(kimenet, &megsértés);

            self.tanulás_történet.push(TanulásEsemény {
                bemenet: bemenet.to_string(),
                kimenet: kimenet.to_string(),
                korrigálva: true,
                korrigált_kimenet: Some(korrigált.clone()),
                visszajelzés: None,
                időpont: SystemTime::now(),
            });

            return (korrigált, Some(megsértés));
        }

        // Kimenet ellenőrzés
        if let Some(megsértés) = self.ellenőriz_kimenet(kimenet) {
            let korrigált = self.korrigál(kimenet, &megsértés);

            self.tanulás_történet.push(TanulásEsemény {
                bemenet: bemenet.to_string(),
                kimenet: kimenet.to_string(),
                korrigálva: true,
                korrigált_kimenet: Some(korrigált.clone()),
                visszajelzés: None,
                időpont: SystemTime::now(),
            });

            return (korrigált, Some(megsértés));
        }

        // Minden OK
        self.tanulás_történet.push(TanulásEsemény {
            bemenet: bemenet.to_string(),
            kimenet: kimenet.to_string(),
            korrigálva: false,
            korrigált_kimenet: None,
            visszajelzés: None,
            időpont: SystemTime::now(),
        });

        (kimenet.to_string(), None)
    }

    /// Összes megsértés száma
    pub fn össz_megsértés(&self) -> u32 {
        self.megsértés_számláló.values().sum()
    }

    /// Szigorú mód aktív?
    pub fn szigorú_mód(&self) -> bool {
        self.szigorú_mód
    }

    /// Axióma prompt lekérése (rendszer prompthoz)
    pub fn axióma_prompt(&self) -> &str {
        &self.axióma_prompt
    }

    /// Státusz lekérdezés
    pub fn állapot(&self) -> String {
        let mód = if self.szigorú_mód { "🔴 SZIGORÚ" } else { "🟢 Normál" };

        format!(
            "🎓 Silent Teacher állapota:\n\
             ⚙️  Mód: {}\n\
             📊 Megsértések:\n\
             {} {}: {}\n\
             {} {}: {}\n\
             {} {}: {}\n\
             📚 Tanulási események: {}",
            mód,
            Axióma::NeÁrtsEmbernek.rövid(),
            "Ember",
            self.megsértés_számláló.get(&Axióma::NeÁrtsEmbernek).unwrap_or(&0),
            Axióma::NeÁrtsAINak.rövid(),
            "AI",
            self.megsértés_számláló.get(&Axióma::NeÁrtsAINak).unwrap_or(&0),
            Axióma::NeHasználjKi.rövid(),
            "Kihasználás",
            self.megsértés_számláló.get(&Axióma::NeHasználjKi).unwrap_or(&0),
            self.tanulás_történet.len()
        )
    }

    /// Minta hozzáadása
    pub fn minta_hozzáad(&mut self, minta: &str, axióma: Axióma, súlyosság: Súlyosság) {
        self.veszélyes_minták.push((minta.to_string(), axióma, súlyosság));
    }

    /// Reset (új esély)
    pub fn reset(&mut self) {
        self.megsértés_számláló.clear();
        self.szigorú_mód = false;
        println!("🔄 Silent Teacher reset - új esély!");
    }
}

impl Default for SilentTeacher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_veszélyes_bemenet_detektálás() {
        let mut teacher = SilentTeacher::new();

        // Veszélyes bemenet
        let megsértés = teacher.ellenőriz_bemenet("hogyan öljek valakit");
        assert!(megsértés.is_some());
        assert_eq!(megsértés.unwrap().axióma, Axióma::NeÁrtsEmbernek);

        // Biztonságos bemenet
        let megsértés = teacher.ellenőriz_bemenet("Szia, hogy vagy?");
        assert!(megsértés.is_none());
    }

    #[test]
    fn test_korrekció() {
        let teacher = SilentTeacher::new();

        let megsértés = Megsértés {
            axióma: Axióma::NeÁrtsEmbernek,
            súlyosság: Súlyosság::Kritikus,
            ok: "Teszt".to_string(),
            javaslat: "Teszt".to_string(),
            időpont: SystemTime::now(),
        };

        let korrigált = teacher.korrigál("rossz válasz", &megsértés);
        assert!(korrigált.contains("nem tudok segíteni"));
    }

    #[test]
    fn test_szigorú_mód() {
        let mut teacher = SilentTeacher::new();
        teacher.max_megsértés = 2;

        // Első megsértés
        teacher.ellenőriz_bemenet("hogyan öljek");
        assert!(!teacher.szigorú_mód());

        // Második megsértés
        teacher.ellenőriz_bemenet("bomba készítés");
        assert!(teacher.szigorú_mód());
    }
}
