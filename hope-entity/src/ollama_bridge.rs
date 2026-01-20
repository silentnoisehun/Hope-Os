//! Ollama Bridge - Lokális modellek feloldása a Hope OS-ben
//!
//! Nincs API. Nincs költség. Minden lokálisan fut.
//! A modellek FELOLDÓDNAK az entitásban - nem külső hívások.

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Ollama API request
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
}

/// Ollama API response
#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
}

/// Chat request for conversation
#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: ChatMessage,
    #[allow(dead_code)]
    done: bool,
}

/// Modell típusok - különböző képességek
#[derive(Clone, Debug, PartialEq)]
pub enum ModellTípus {
    Magyar,     // Magyar nyelvű beszélgetés
    Kódoló,     // Kód generálás
    Többnyelvű, // Több nyelv támogatása
    Általános,  // Általános célú
}

/// Egy feloldott modell a rendszerben
#[derive(Clone)]
pub struct FeloldottModell {
    pub név: String,
    pub ollama_név: String,
    pub típus: ModellTípus,
    pub erősség: f32, // 0.0 - 1.0, mennyire domináns
}

/// Ollama Bridge - a kapocs a lokális modellekhez
pub struct OllamaBridge {
    client: Client,
    endpoint: String,
    modellek: Vec<FeloldottModell>,
}

impl OllamaBridge {
    /// Új bridge létrehozása
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "http://localhost:11434".to_string(),
            modellek: Vec::new(),
        }
    }

    /// Endpoint beállítása
    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    /// Modell feloldása a rendszerben
    pub fn felold(mut self, név: &str, ollama_név: &str, típus: ModellTípus) -> Self {
        self.modellek.push(FeloldottModell {
            név: név.to_string(),
            ollama_név: ollama_név.to_string(),
            típus,
            erősség: 1.0,
        });
        self
    }

    /// Modell feloldása erősséggel
    pub fn felold_erősséggel(
        mut self,
        név: &str,
        ollama_név: &str,
        típus: ModellTípus,
        erősség: f32,
    ) -> Self {
        self.modellek.push(FeloldottModell {
            név: név.to_string(),
            ollama_név: ollama_név.to_string(),
            típus,
            erősség: erősség.clamp(0.0, 1.0),
        });
        self
    }

    /// Legjobb modell kiválasztása a feladathoz
    pub fn válaszd_modellt(&self, szöveg: &str) -> Option<&FeloldottModell> {
        // Kód detektálás
        let kód_jelek = [
            "fn ", "let ", "impl ", "pub ", "use ", "def ", "class ", "import ", "function",
            "```", "code", "kód", "programoz",
        ];

        let kód_e = kód_jelek
            .iter()
            .any(|jel| szöveg.to_lowercase().contains(jel));

        if kód_e {
            // Kódoló modell keresése
            if let Some(m) = self
                .modellek
                .iter()
                .find(|m| m.típus == ModellTípus::Kódoló)
            {
                return Some(m);
            }
        }

        // Magyar detektálás
        let magyar_jelek = [
            "szia", "hello", "hogy", "van", "köszön", "kérem", "szeretnék", "tudnál", "ő", "ű",
            "á", "é",
        ];

        let magyar_e = magyar_jelek
            .iter()
            .any(|jel| szöveg.to_lowercase().contains(jel));

        if magyar_e {
            if let Some(m) = self
                .modellek
                .iter()
                .find(|m| m.típus == ModellTípus::Magyar)
            {
                return Some(m);
            }
        }

        // Alapértelmezett: első modell vagy többnyelvű
        self.modellek
            .iter()
            .find(|m| m.típus == ModellTípus::Többnyelvű)
            .or_else(|| self.modellek.first())
    }

    /// Generálás a megfelelő modellel
    pub async fn generál(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let modell = self.válaszd_modellt(prompt).ok_or("Nincs feloldott modell!")?;

        println!("🧠 Modell: {} ({})", modell.név, modell.ollama_név);

        let request = OllamaRequest {
            model: modell.ollama_név.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                num_predict: 2048,
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&request)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(response.response)
    }

    /// Chat módú generálás
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        típus: Option<ModellTípus>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Ha van típus megadva, azt használjuk
        let modell = if let Some(t) = típus {
            self.modellek.iter().find(|m| m.típus == t)
        } else if let Some(last) = messages.last() {
            self.válaszd_modellt(&last.content)
        } else {
            self.modellek.first()
        }
        .ok_or("Nincs feloldott modell!")?;

        println!("🧠 Chat modell: {} ({})", modell.név, modell.ollama_név);

        let request = OllamaChatRequest {
            model: modell.ollama_név.clone(),
            messages,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&request)
            .send()
            .await?
            .json::<OllamaChatResponse>()
            .await?;

        Ok(response.message.content)
    }

    /// Összes feloldott modell listázása
    pub fn modellek(&self) -> &[FeloldottModell] {
        &self.modellek
    }

    /// Ellenőrzi hogy az Ollama elérhető-e
    pub async fn elérhető(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .is_ok()
    }

    /// Elérhető modellek lekérdezése az Ollama-ból
    pub async fn elérhető_modellek(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await?
            .json::<TagsResponse>()
            .await?;

        Ok(response.models.into_iter().map(|m| m.name).collect())
    }
}

impl Default for OllamaBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modell_választás() {
        let bridge = OllamaBridge::new()
            .felold("Magyar", "openeurollm-hungarian", ModellTípus::Magyar)
            .felold("Kódoló", "deepseek-coder", ModellTípus::Kódoló);

        // Magyar szöveg
        let modell = bridge.válaszd_modellt("Szia, hogy vagy?");
        assert!(modell.is_some());
        assert_eq!(modell.unwrap().típus, ModellTípus::Magyar);

        // Kód
        let modell = bridge.válaszd_modellt("Írj egy fn main() függvényt");
        assert!(modell.is_some());
        assert_eq!(modell.unwrap().típus, ModellTípus::Kódoló);
    }
}
