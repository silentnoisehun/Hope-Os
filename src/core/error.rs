//! Hope OS - Hibakezelés
//!
//! Központi hibatípusok a rendszer számára.

use thiserror::Error;

/// Hope OS hibatípusok
#[derive(Error, Debug)]
pub enum HopeError {
    #[error("Modul hiba: {0}")]
    Module(String),

    #[error("Regisztráció hiba: {0}")]
    Registration(String),

    #[error("Nem található: {0}")]
    NotFound(String),

    #[error("Graf hiba: {0}")]
    Graph(String),

    #[error("gRPC hiba: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("gRPC transport hiba: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("IO hiba: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON hiba: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP hiba: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Voice hiba: {0}")]
    Voice(String),

    #[error("Általános hiba: {0}")]
    General(String),
}

// From<String> implementáció az egyszerű hibakezeléshez
impl From<String> for HopeError {
    fn from(s: String) -> Self {
        HopeError::General(s)
    }
}

impl From<&str> for HopeError {
    fn from(s: &str) -> Self {
        HopeError::General(s.to_string())
    }
}

/// Hope eredmény típus
pub type HopeResult<T> = Result<T, HopeError>;
