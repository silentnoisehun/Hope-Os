//! Hope OS - Geolocation System
//!
//! Hope tudja HOVA tartoznak az emlékek. Térbeli kontextus minden memóriához.
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Földrajzi koordináta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Szélesség (latitude) - -90 to 90
    pub latitude: f64,
    /// Hosszúság (longitude) - -180 to 180
    pub longitude: f64,
    /// Magasság méterben (opcionális)
    pub altitude: Option<f64>,
    /// Pontosság méterben
    pub accuracy: Option<f64>,
    /// Időbélyeg
    pub timestamp: DateTime<Utc>,
    /// Forrás (gps, ip, manual, inferred)
    pub source: GeoSource,
}

/// Lokáció forrás típusa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeoSource {
    /// GPS/GNSS adat
    Gps,
    /// IP cím alapú
    IpBased,
    /// Manuálisan megadott
    Manual,
    /// Következtetett (pl. emlékekből)
    Inferred,
    /// WiFi/Bluetooth alapú
    Network,
    /// Ismeretlen
    Unknown,
}

impl Default for GeoSource {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Hely (ismert lokáció)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    /// Egyedi azonosító
    pub id: Uuid,
    /// Hely neve
    pub name: String,
    /// Hely típusa
    pub place_type: PlaceType,
    /// Központi koordináta
    pub location: GeoLocation,
    /// Sugár méterben (a hely kiterjedése)
    pub radius: f64,
    /// Cím (ha ismert)
    pub address: Option<String>,
    /// Ország kód (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
    /// Látogatások száma
    pub visit_count: u64,
    /// Utolsó látogatás
    pub last_visit: Option<DateTime<Utc>>,
    /// Első látogatás
    pub first_visit: DateTime<Utc>,
    /// Kapcsolódó emlékek száma
    pub memory_count: u64,
    /// Érzelmi asszociációk (érzelem -> intenzitás)
    pub emotional_associations: HashMap<String, f64>,
    /// Egyéb metaadatok
    pub metadata: HashMap<String, String>,
}

/// Hely típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceType {
    /// Otthon
    Home,
    /// Munkahely
    Work,
    /// Iskola/egyetem
    School,
    /// Bolt/bevásárlóközpont
    Shopping,
    /// Étterem/kávézó
    Restaurant,
    /// Park/természet
    Nature,
    /// Orvos/kórház
    Medical,
    /// Szórakozóhely
    Entertainment,
    /// Közlekedési csomópont
    Transit,
    /// Barát/rokon háza
    Social,
    /// Egyéb
    Other,
}

impl Default for PlaceType {
    fn default() -> Self {
        Self::Other
    }
}

impl std::fmt::Display for PlaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceType::Home => write!(f, "home"),
            PlaceType::Work => write!(f, "work"),
            PlaceType::School => write!(f, "school"),
            PlaceType::Shopping => write!(f, "shopping"),
            PlaceType::Restaurant => write!(f, "restaurant"),
            PlaceType::Nature => write!(f, "nature"),
            PlaceType::Medical => write!(f, "medical"),
            PlaceType::Entertainment => write!(f, "entertainment"),
            PlaceType::Transit => write!(f, "transit"),
            PlaceType::Social => write!(f, "social"),
            PlaceType::Other => write!(f, "other"),
        }
    }
}

impl PlaceType {
    /// String-ből konvertál
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "home" | "otthon" => PlaceType::Home,
            "work" | "munka" | "munkahely" => PlaceType::Work,
            "school" | "iskola" | "egyetem" => PlaceType::School,
            "shopping" | "bolt" | "áruház" => PlaceType::Shopping,
            "restaurant" | "étterem" | "kávézó" => PlaceType::Restaurant,
            "nature" | "park" | "természet" => PlaceType::Nature,
            "medical" | "orvos" | "kórház" => PlaceType::Medical,
            "entertainment" | "szórakozás" | "mozi" => PlaceType::Entertainment,
            "transit" | "állomás" | "megálló" => PlaceType::Transit,
            "social" | "barát" | "rokon" => PlaceType::Social,
            _ => PlaceType::Other,
        }
    }
}

/// Földrajzi emlék - egy emlékhez tartozó lokáció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoMemory {
    /// Emlék ID (a memory rendszerből)
    pub memory_id: Uuid,
    /// Lokáció
    pub location: GeoLocation,
    /// Hely ID (ha ismert helyhez tartozik)
    pub place_id: Option<Uuid>,
    /// Hely neve (ha ismert)
    pub place_name: Option<String>,
    /// Létrehozás időpontja
    pub created_at: DateTime<Utc>,
    /// Fontosság (0.0 - 1.0)
    pub importance: f64,
}

/// Privacy beállítások
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPrivacySettings {
    /// Lokáció követés engedélyezve
    pub tracking_enabled: bool,
    /// Pontos lokáció tárolása
    pub store_precise_location: bool,
    /// Lokáció homályosítás sugara (méter)
    pub blur_radius: f64,
    /// Bizonyos helyek rejtése
    pub hidden_places: Vec<Uuid>,
    /// Lokáció megosztás engedélyezve
    pub sharing_enabled: bool,
    /// Lokáció történet megőrzési idő (napok, 0 = örökké)
    pub retention_days: u32,
}

impl Default for GeoPrivacySettings {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            store_precise_location: false, // Alapból homályosít
            blur_radius: 100.0,            // 100 méter
            hidden_places: Vec::new(),
            sharing_enabled: false,
            retention_days: 365, // 1 év
        }
    }
}

/// Geo Engine statisztikák
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoStats {
    /// Összes tárolt lokáció
    pub total_locations: u64,
    /// Összes ismert hely
    pub total_places: u64,
    /// Összes geo-emlék
    pub total_geo_memories: u64,
    /// Otthon beállítva
    pub home_set: bool,
    /// Munkahely beállítva
    pub work_set: bool,
    /// Utoljára frissítve
    pub last_update: Option<DateTime<Utc>>,
    /// Összes megtett távolság (km)
    pub total_distance_km: f64,
}

/// Geolocation Engine - a fő motor
pub struct GeoEngine {
    /// Ismert helyek
    places: Arc<RwLock<HashMap<Uuid, Place>>>,
    /// Lokáció történet
    location_history: Arc<RwLock<Vec<GeoLocation>>>,
    /// Geo-emlékek
    geo_memories: Arc<RwLock<Vec<GeoMemory>>>,
    /// Jelenlegi lokáció
    current_location: Arc<RwLock<Option<GeoLocation>>>,
    /// Otthon ID
    home_id: Arc<RwLock<Option<Uuid>>>,
    /// Munkahely ID
    work_id: Arc<RwLock<Option<Uuid>>>,
    /// Privacy beállítások
    privacy: Arc<RwLock<GeoPrivacySettings>>,
    /// Statisztikák
    stats: Arc<RwLock<GeoStats>>,
}

impl GeoEngine {
    /// Új GeoEngine létrehozása
    pub fn new() -> Self {
        Self {
            places: Arc::new(RwLock::new(HashMap::new())),
            location_history: Arc::new(RwLock::new(Vec::new())),
            geo_memories: Arc::new(RwLock::new(Vec::new())),
            current_location: Arc::new(RwLock::new(None)),
            home_id: Arc::new(RwLock::new(None)),
            work_id: Arc::new(RwLock::new(None)),
            privacy: Arc::new(RwLock::new(GeoPrivacySettings::default())),
            stats: Arc::new(RwLock::new(GeoStats {
                total_locations: 0,
                total_places: 0,
                total_geo_memories: 0,
                home_set: false,
                work_set: false,
                last_update: None,
                total_distance_km: 0.0,
            })),
        }
    }

    /// Haversine formula - két pont közötti távolság (km)
    pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS_KM * c
    }

    /// Távolság két GeoLocation között (km)
    pub fn distance(loc1: &GeoLocation, loc2: &GeoLocation) -> f64 {
        Self::haversine_distance(loc1.latitude, loc1.longitude, loc2.latitude, loc2.longitude)
    }

    /// Jelenlegi lokáció beállítása
    pub async fn set_current_location(&self, location: GeoLocation) -> Result<(), String> {
        let privacy = self.privacy.read().await;
        if !privacy.tracking_enabled {
            return Err("Location tracking is disabled".to_string());
        }

        // Távolság számítás előző lokációtól
        let distance_km = {
            let current = self.current_location.read().await;
            if let Some(ref curr) = *current {
                Self::distance(curr, &location)
            } else {
                0.0
            }
        };

        // Lokáció tárolása (esetleg homályosítva)
        let stored_location = if privacy.store_precise_location {
            location.clone()
        } else {
            self.blur_location(&location, privacy.blur_radius)
        };
        drop(privacy);

        // Aktuális lokáció frissítése
        {
            let mut current = self.current_location.write().await;
            *current = Some(stored_location.clone());
        }

        // Történethez hozzáadás
        {
            let mut history = self.location_history.write().await;
            history.push(stored_location);
        }

        // Statisztikák frissítése
        {
            let mut stats = self.stats.write().await;
            stats.total_locations += 1;
            stats.total_distance_km += distance_km;
            stats.last_update = Some(Utc::now());
        }

        // Hely detektálás
        self.detect_place(&location).await;

        Ok(())
    }

    /// Lokáció homályosítása privacy miatt
    fn blur_location(&self, location: &GeoLocation, radius_meters: f64) -> GeoLocation {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Random offset a sugáron belül
        let angle = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
        let distance = rng.gen::<f64>() * radius_meters;

        // Méterből fokra konvertálás (közelítés)
        let lat_offset = (distance * angle.cos()) / 111_111.0;
        let lon_offset =
            (distance * angle.sin()) / (111_111.0 * location.latitude.to_radians().cos());

        GeoLocation {
            latitude: location.latitude + lat_offset,
            longitude: location.longitude + lon_offset,
            altitude: location.altitude,
            accuracy: Some(radius_meters),
            timestamp: location.timestamp,
            source: location.source,
        }
    }

    /// Jelenlegi lokáció lekérdezése
    pub async fn get_current_location(&self) -> Option<GeoLocation> {
        self.current_location.read().await.clone()
    }

    /// Hely hozzáadása
    pub async fn add_place(&self, mut place: Place) -> Uuid {
        place.id = Uuid::new_v4();
        place.first_visit = Utc::now();
        place.visit_count = 0;
        place.memory_count = 0;

        let id = place.id;

        {
            let mut places = self.places.write().await;
            places.insert(id, place);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_places += 1;
        }

        id
    }

    /// Otthon beállítása
    pub async fn set_home(&self, place_id: Uuid) -> Result<(), String> {
        let places = self.places.read().await;
        if !places.contains_key(&place_id) {
            return Err("Place not found".to_string());
        }
        drop(places);

        {
            let mut home = self.home_id.write().await;
            *home = Some(place_id);
        }

        // Hely típus frissítése
        {
            let mut places = self.places.write().await;
            if let Some(place) = places.get_mut(&place_id) {
                place.place_type = PlaceType::Home;
            }
        }

        {
            let mut stats = self.stats.write().await;
            stats.home_set = true;
        }

        Ok(())
    }

    /// Munkahely beállítása
    pub async fn set_work(&self, place_id: Uuid) -> Result<(), String> {
        let places = self.places.read().await;
        if !places.contains_key(&place_id) {
            return Err("Place not found".to_string());
        }
        drop(places);

        {
            let mut work = self.work_id.write().await;
            *work = Some(place_id);
        }

        // Hely típus frissítése
        {
            let mut places = self.places.write().await;
            if let Some(place) = places.get_mut(&place_id) {
                place.place_type = PlaceType::Work;
            }
        }

        {
            let mut stats = self.stats.write().await;
            stats.work_set = true;
        }

        Ok(())
    }

    /// Otthon lekérdezése
    pub async fn get_home(&self) -> Option<Place> {
        let home_id = self.home_id.read().await;
        if let Some(id) = *home_id {
            let places = self.places.read().await;
            places.get(&id).cloned()
        } else {
            None
        }
    }

    /// Munkahely lekérdezése
    pub async fn get_work(&self) -> Option<Place> {
        let work_id = self.work_id.read().await;
        if let Some(id) = *work_id {
            let places = self.places.read().await;
            places.get(&id).cloned()
        } else {
            None
        }
    }

    /// Hely detektálás az aktuális lokáció alapján
    async fn detect_place(&self, location: &GeoLocation) {
        // Először megkeressük a helyet (csak olvasás)
        let found = {
            let places = self.places.read().await;
            let mut found_place: Option<(Uuid, String)> = None;

            for (id, place) in places.iter() {
                let distance_km = Self::distance(location, &place.location);
                let distance_m = distance_km * 1000.0;

                if distance_m <= place.radius {
                    found_place = Some((*id, place.name.clone()));
                    break;
                }
            }

            found_place
        };

        // Ha találtunk helyet, frissítjük (írás)
        if let Some((place_id, place_name)) = found {
            let mut places = self.places.write().await;
            if let Some(p) = places.get_mut(&place_id) {
                p.visit_count += 1;
                p.last_visit = Some(Utc::now());
            }

            tracing::debug!(place_name = %place_name, "Detected place visit");
        }
    }

    /// Helyek listázása
    pub async fn list_places(&self) -> Vec<Place> {
        self.places.read().await.values().cloned().collect()
    }

    /// Helyek keresése típus alapján
    pub async fn find_places_by_type(&self, place_type: PlaceType) -> Vec<Place> {
        self.places
            .read()
            .await
            .values()
            .filter(|p| p.place_type == place_type)
            .cloned()
            .collect()
    }

    /// Közeli helyek keresése
    pub async fn find_nearby_places(&self, location: &GeoLocation, radius_km: f64) -> Vec<Place> {
        self.places
            .read()
            .await
            .values()
            .filter(|p| Self::distance(location, &p.location) <= radius_km)
            .cloned()
            .collect()
    }

    /// Hely lekérdezése ID alapján
    pub async fn get_place(&self, place_id: Uuid) -> Option<Place> {
        self.places.read().await.get(&place_id).cloned()
    }

    /// Geo-emlék hozzáadása
    pub async fn add_geo_memory(&self, memory_id: Uuid, location: GeoLocation, importance: f64) {
        // Megkeressük, melyik helyhez tartozik
        let found = {
            let places = self.places.read().await;
            let mut found: Option<(Uuid, String)> = None;

            for (id, place) in places.iter() {
                let distance_km = Self::distance(&location, &place.location);
                let distance_m = distance_km * 1000.0;

                if distance_m <= place.radius {
                    found = Some((*id, place.name.clone()));
                    break;
                }
            }

            found
        };

        let (place_id, place_name) = match found {
            Some((id, name)) => (Some(id), Some(name)),
            None => (None, None),
        };

        // Memory count növelése
        if let Some(pid) = place_id {
            let mut places = self.places.write().await;
            if let Some(p) = places.get_mut(&pid) {
                p.memory_count += 1;
            }
        }

        let geo_memory = GeoMemory {
            memory_id,
            location,
            place_id,
            place_name: place_name.clone(),
            created_at: Utc::now(),
            importance,
        };

        {
            let mut memories = self.geo_memories.write().await;
            memories.push(geo_memory);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_geo_memories += 1;
        }
    }

    /// Geo-emlékek lekérdezése helyhez
    pub async fn get_memories_at_place(&self, place_id: Uuid) -> Vec<GeoMemory> {
        self.geo_memories
            .read()
            .await
            .iter()
            .filter(|m| m.place_id == Some(place_id))
            .cloned()
            .collect()
    }

    /// Geo-emlékek lekérdezése sugáron belül
    pub async fn get_memories_nearby(
        &self,
        location: &GeoLocation,
        radius_km: f64,
    ) -> Vec<GeoMemory> {
        self.geo_memories
            .read()
            .await
            .iter()
            .filter(|m| Self::distance(&m.location, location) <= radius_km)
            .cloned()
            .collect()
    }

    /// Távolság az otthontól
    pub async fn distance_from_home(&self, location: &GeoLocation) -> Option<f64> {
        if let Some(home) = self.get_home().await {
            Some(Self::distance(location, &home.location))
        } else {
            None
        }
    }

    /// Távolság a munkahelytől
    pub async fn distance_from_work(&self, location: &GeoLocation) -> Option<f64> {
        if let Some(work) = self.get_work().await {
            Some(Self::distance(location, &work.location))
        } else {
            None
        }
    }

    /// Távolság két hely között
    pub async fn distance_between_places(&self, place1_id: Uuid, place2_id: Uuid) -> Option<f64> {
        let places = self.places.read().await;
        let p1 = places.get(&place1_id)?;
        let p2 = places.get(&place2_id)?;
        Some(Self::distance(&p1.location, &p2.location))
    }

    /// Privacy beállítások lekérdezése
    pub async fn get_privacy_settings(&self) -> GeoPrivacySettings {
        self.privacy.read().await.clone()
    }

    /// Privacy beállítások módosítása
    pub async fn update_privacy_settings(&self, settings: GeoPrivacySettings) {
        let mut privacy = self.privacy.write().await;
        *privacy = settings;
    }

    /// Statisztikák lekérdezése
    pub async fn get_stats(&self) -> GeoStats {
        self.stats.read().await.clone()
    }

    /// Összes lokáció történet
    pub async fn get_location_history(&self, limit: usize) -> Vec<GeoLocation> {
        let history = self.location_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Lokáció történet törlése
    pub async fn clear_location_history(&self) {
        let mut history = self.location_history.write().await;
        history.clear();

        let mut stats = self.stats.write().await;
        stats.total_locations = 0;
        stats.total_distance_km = 0.0;
    }

    /// Összes adat törlése
    pub async fn clear_all(&self) {
        {
            let mut places = self.places.write().await;
            places.clear();
        }
        {
            let mut history = self.location_history.write().await;
            history.clear();
        }
        {
            let mut memories = self.geo_memories.write().await;
            memories.clear();
        }
        {
            let mut current = self.current_location.write().await;
            *current = None;
        }
        {
            let mut home = self.home_id.write().await;
            *home = None;
        }
        {
            let mut work = self.work_id.write().await;
            *work = None;
        }
        {
            let mut stats = self.stats.write().await;
            *stats = GeoStats {
                total_locations: 0,
                total_places: 0,
                total_geo_memories: 0,
                home_set: false,
                work_set: false,
                last_update: None,
                total_distance_km: 0.0,
            };
        }
    }
}

impl Default for GeoEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        // Budapest -> Bécs kb. 215 km
        let distance = GeoEngine::haversine_distance(47.4979, 19.0402, 48.2082, 16.3738);
        assert!((distance - 215.0).abs() < 20.0); // ±20 km tolerance

        // Ugyanaz a pont -> 0
        let same = GeoEngine::haversine_distance(47.4979, 19.0402, 47.4979, 19.0402);
        assert!(same < 0.001);
    }

    #[test]
    fn test_geo_location() {
        let loc = GeoLocation {
            latitude: 47.4979,
            longitude: 19.0402,
            altitude: Some(120.0),
            accuracy: Some(10.0),
            timestamp: Utc::now(),
            source: GeoSource::Gps,
        };

        assert!((loc.latitude - 47.4979).abs() < 0.0001);
        assert_eq!(loc.source, GeoSource::Gps);
    }

    #[test]
    fn test_place_type_from_str() {
        assert_eq!(PlaceType::from_str("home"), PlaceType::Home);
        assert_eq!(PlaceType::from_str("otthon"), PlaceType::Home);
        assert_eq!(PlaceType::from_str("work"), PlaceType::Work);
        assert_eq!(PlaceType::from_str("munka"), PlaceType::Work);
        assert_eq!(PlaceType::from_str("unknown"), PlaceType::Other);
    }

    #[tokio::test]
    async fn test_geo_engine_new() {
        let engine = GeoEngine::new();
        let stats = engine.get_stats().await;

        assert_eq!(stats.total_locations, 0);
        assert_eq!(stats.total_places, 0);
        assert!(!stats.home_set);
    }

    #[tokio::test]
    async fn test_add_place() {
        let engine = GeoEngine::new();

        let place = Place {
            id: Uuid::nil(),
            name: "Test Place".to_string(),
            place_type: PlaceType::Home,
            location: GeoLocation {
                latitude: 47.4979,
                longitude: 19.0402,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 100.0,
            address: Some("Budapest, Hungary".to_string()),
            country_code: Some("HU".to_string()),
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let id = engine.add_place(place).await;
        assert_ne!(id, Uuid::nil());

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_places, 1);

        let retrieved = engine.get_place(id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Place");
    }

    #[tokio::test]
    async fn test_set_home() {
        let engine = GeoEngine::new();

        let place = Place {
            id: Uuid::nil(),
            name: "My Home".to_string(),
            place_type: PlaceType::Other,
            location: GeoLocation {
                latitude: 47.4979,
                longitude: 19.0402,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 50.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let id = engine.add_place(place).await;
        engine.set_home(id).await.unwrap();

        let home = engine.get_home().await;
        assert!(home.is_some());
        assert_eq!(home.unwrap().place_type, PlaceType::Home);

        let stats = engine.get_stats().await;
        assert!(stats.home_set);
    }

    #[tokio::test]
    async fn test_current_location() {
        let engine = GeoEngine::new();

        let loc = GeoLocation {
            latitude: 47.4979,
            longitude: 19.0402,
            altitude: None,
            accuracy: Some(10.0),
            timestamp: Utc::now(),
            source: GeoSource::Gps,
        };

        engine.set_current_location(loc.clone()).await.unwrap();

        let current = engine.get_current_location().await;
        assert!(current.is_some());

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_locations, 1);
    }

    #[tokio::test]
    async fn test_privacy_settings() {
        let engine = GeoEngine::new();

        let mut settings = engine.get_privacy_settings().await;
        assert!(settings.tracking_enabled);
        assert!(!settings.store_precise_location);

        settings.tracking_enabled = false;
        engine.update_privacy_settings(settings.clone()).await;

        let loc = GeoLocation {
            latitude: 47.4979,
            longitude: 19.0402,
            altitude: None,
            accuracy: None,
            timestamp: Utc::now(),
            source: GeoSource::Gps,
        };

        let result = engine.set_current_location(loc).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_geo_memory() {
        let engine = GeoEngine::new();

        let place = Place {
            id: Uuid::nil(),
            name: "Memory Place".to_string(),
            place_type: PlaceType::Nature,
            location: GeoLocation {
                latitude: 47.5,
                longitude: 19.0,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 500.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let place_id = engine.add_place(place).await;

        let memory_loc = GeoLocation {
            latitude: 47.5001,
            longitude: 19.0001,
            altitude: None,
            accuracy: None,
            timestamp: Utc::now(),
            source: GeoSource::Manual,
        };

        let memory_id = Uuid::new_v4();
        engine.add_geo_memory(memory_id, memory_loc, 0.8).await;

        let memories = engine.get_memories_at_place(place_id).await;
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].memory_id, memory_id);

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_geo_memories, 1);
    }

    #[tokio::test]
    async fn test_find_nearby_places() {
        let engine = GeoEngine::new();

        let place1 = Place {
            id: Uuid::nil(),
            name: "Nearby".to_string(),
            place_type: PlaceType::Restaurant,
            location: GeoLocation {
                latitude: 47.5,
                longitude: 19.0,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 100.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let place2 = Place {
            id: Uuid::nil(),
            name: "Far Away".to_string(),
            place_type: PlaceType::Shopping,
            location: GeoLocation {
                latitude: 48.0, // ~55 km away
                longitude: 19.0,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 100.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        engine.add_place(place1).await;
        engine.add_place(place2).await;

        let search_loc = GeoLocation {
            latitude: 47.501,
            longitude: 19.001,
            altitude: None,
            accuracy: None,
            timestamp: Utc::now(),
            source: GeoSource::Manual,
        };

        let nearby = engine.find_nearby_places(&search_loc, 5.0).await; // 5 km radius
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].name, "Nearby");
    }

    #[tokio::test]
    async fn test_distance_calculations() {
        let engine = GeoEngine::new();

        let home_place = Place {
            id: Uuid::nil(),
            name: "Home".to_string(),
            place_type: PlaceType::Home,
            location: GeoLocation {
                latitude: 47.5,
                longitude: 19.0,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 50.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let home_id = engine.add_place(home_place).await;
        engine.set_home(home_id).await.unwrap();

        let current = GeoLocation {
            latitude: 47.6, // ~11 km north
            longitude: 19.0,
            altitude: None,
            accuracy: None,
            timestamp: Utc::now(),
            source: GeoSource::Gps,
        };

        let distance = engine.distance_from_home(&current).await;
        assert!(distance.is_some());
        assert!((distance.unwrap() - 11.1).abs() < 1.0); // ~11 km ± 1 km
    }

    #[tokio::test]
    async fn test_clear_all() {
        let engine = GeoEngine::new();

        let place = Place {
            id: Uuid::nil(),
            name: "Test".to_string(),
            place_type: PlaceType::Other,
            location: GeoLocation {
                latitude: 47.5,
                longitude: 19.0,
                altitude: None,
                accuracy: None,
                timestamp: Utc::now(),
                source: GeoSource::Manual,
            },
            radius: 100.0,
            address: None,
            country_code: None,
            visit_count: 0,
            last_visit: None,
            first_visit: Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        engine.add_place(place).await;

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_places, 1);

        engine.clear_all().await;

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_places, 0);
        assert_eq!(stats.total_locations, 0);
    }
}
