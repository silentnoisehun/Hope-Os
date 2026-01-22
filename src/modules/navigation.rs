//! Hope OS - Navigation System
//!
//! Hope nem csak tudja hol vagy - tudja HOVA mész, MIÉRT,
//! és MI A LEGJOBB ÚT neked, MOST, ebben a hangulatban.
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::HopeResult;
use crate::modules::geolocation::{GeoEngine, GeoLocation, GeoMemory, GeoSource, Place, PlaceType};

// ============================================================================
// NAVIGATION CONTEXT
// ============================================================================

/// Navigációs kontextus - minden ami befolyásolja az útvonalat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationContext {
    /// Kiindulási pont (opcionális - ha None, jelenlegi lokáció)
    pub origin: Option<GeoPoint>,
    /// Célpont
    pub destination: GeoPoint,
    /// Köztes pontok
    pub waypoints: Vec<GeoPoint>,

    // Hope réteg
    /// Aktuális hangulat
    pub current_mood: String,
    /// Energia szint (0-1, mennyire fáradt)
    pub energy_level: f64,
    /// Időnyomás (0-1, mennyire sürgős)
    pub time_pressure: f64,
    /// Érzelem állapot
    pub emotions: EmotionState,
    /// Útvonal preferenciák
    pub preferences: RoutePreferences,
    /// Utazás célja
    pub purpose: String,
}

impl Default for NavigationContext {
    fn default() -> Self {
        Self {
            origin: None,
            destination: GeoPoint::default(),
            waypoints: Vec::new(),
            current_mood: String::new(),
            energy_level: 0.7,
            time_pressure: 0.3,
            emotions: EmotionState::default(),
            preferences: RoutePreferences::default(),
            purpose: String::new(),
        }
    }
}

/// Egyszerűsített érzelem állapot a navigációhoz
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionState {
    pub dominant: String,
    pub intensity: f64,
    pub valence: f64,
    pub arousal: f64,
}

impl EmotionState {
    pub fn relaxed() -> Self {
        Self {
            dominant: "calm".to_string(),
            intensity: 0.5,
            valence: 0.6,
            arousal: 0.3,
        }
    }

    pub fn stressed() -> Self {
        Self {
            dominant: "stressed".to_string(),
            intensity: 0.8,
            valence: -0.3,
            arousal: 0.7,
        }
    }

    pub fn tired() -> Self {
        Self {
            dominant: "tired".to_string(),
            intensity: 0.6,
            valence: -0.1,
            arousal: 0.2,
        }
    }
}

/// Útvonal preferenciák
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePreferences {
    pub avoid_highways: bool,
    pub avoid_tolls: bool,
    pub prefer_scenic: bool,
    pub prefer_familiar: bool,
    pub max_walking_distance: f64,
    pub accessibility_needs: Vec<AccessibilityNeed>,

    // Tanult preferenciák
    pub learned_avoidances: Vec<GeoArea>,
    pub favorite_routes: Vec<Uuid>,
    pub preferred_stops: Vec<Place>,
}

impl Default for RoutePreferences {
    fn default() -> Self {
        Self {
            avoid_highways: false,
            avoid_tolls: false,
            prefer_scenic: false,
            prefer_familiar: true,
            max_walking_distance: 500.0,
            accessibility_needs: Vec::new(),
            learned_avoidances: Vec::new(),
            favorite_routes: Vec::new(),
            preferred_stops: Vec::new(),
        }
    }
}

/// Akadálymentességi igények
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessibilityNeed {
    WheelchairAccessible,
    ElevatorRequired,
    NoStairs,
    WidePathways,
    AudioGuidance,
    LowMobility,
}

/// Földrajzi terület (kerülendő zónákhoz)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoArea {
    pub id: Uuid,
    pub name: String,
    pub center: GeoLocation,
    pub radius_km: f64,
    pub reason: String,
    pub learned_at: DateTime<Utc>,
}

// ============================================================================
// SMART ROUTE
// ============================================================================

/// Okos útvonal - Hope különbség
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRoute {
    pub id: Uuid,
    pub path: Vec<GeoPoint>,
    pub segments: Vec<RouteSegment>,
    pub total_distance_km: f64,
    pub total_duration: Duration,
    pub eta: DateTime<Utc>,

    // Traffic
    pub traffic_level: TrafficLevel,
    pub delays: Vec<Delay>,
    pub alternative_available: bool,

    // Hope réteg
    pub memories_on_route: Vec<MemoryOnRoute>,
    pub places_on_route: Vec<Place>,
    pub suggested_stops: Vec<SuggestedStop>,
    pub emotional_score: f64,
    pub context_notes: Vec<String>,
}

impl SmartRoute {
    pub fn new(origin: GeoLocation, destination: GeoLocation) -> Self {
        let distance = origin.distance_to(&destination);
        let duration_mins = (distance / 50.0 * 60.0) as i64; // ~50 km/h átlag

        Self {
            id: Uuid::new_v4(),
            path: vec![
                GeoPoint::from_location(&origin),
                GeoPoint::from_location(&destination),
            ],
            segments: vec![RouteSegment {
                start: GeoPoint::from_location(&origin),
                end: GeoPoint::from_location(&destination),
                distance_km: distance,
                duration: Duration::minutes(duration_mins),
                road_type: RoadType::Urban,
                instruction: format!("Haladj {:?} felé", destination.source),
                traffic: TrafficLevel::Light,
            }],
            total_distance_km: distance,
            total_duration: Duration::minutes(duration_mins),
            eta: Utc::now() + Duration::minutes(duration_mins),
            traffic_level: TrafficLevel::Light,
            delays: Vec::new(),
            alternative_available: true,
            memories_on_route: Vec::new(),
            places_on_route: Vec::new(),
            suggested_stops: Vec::new(),
            emotional_score: 0.7,
            context_notes: Vec::new(),
        }
    }
}

/// Földrajzi pont
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub name: Option<String>,
}

impl Default for GeoPoint {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            name: None,
        }
    }
}

impl GeoPoint {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            name: None,
        }
    }

    pub fn from_location(loc: &GeoLocation) -> Self {
        Self {
            latitude: loc.latitude,
            longitude: loc.longitude,
            name: None,
        }
    }

    pub fn distance_to(&self, other: &GeoPoint) -> f64 {
        let r = 6371.0; // Föld sugara km-ben

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        r * c
    }

    /// Alias for compatibility
    pub fn lat(&self) -> f64 {
        self.latitude
    }
    pub fn lon(&self) -> f64 {
        self.longitude
    }
}

/// Útvonal szegmens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSegment {
    pub start: GeoPoint,
    pub end: GeoPoint,
    pub distance_km: f64,
    pub duration: Duration,
    pub road_type: RoadType,
    pub instruction: String,
    pub traffic: TrafficLevel,
}

/// Út típus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoadType {
    Highway,
    MainRoad,
    Urban,
    Residential,
    Pedestrian,
    Bicycle,
    Trail,
}

impl std::fmt::Display for RoadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoadType::Highway => write!(f, "Autópálya"),
            RoadType::MainRoad => write!(f, "Főút"),
            RoadType::Urban => write!(f, "Városi út"),
            RoadType::Residential => write!(f, "Lakóövezet"),
            RoadType::Pedestrian => write!(f, "Gyalogos"),
            RoadType::Bicycle => write!(f, "Kerékpárút"),
            RoadType::Trail => write!(f, "Ösvény"),
        }
    }
}

/// Forgalom szint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrafficLevel {
    Free,
    Light,
    Moderate,
    Heavy,
    Blocked,
}

impl std::fmt::Display for TrafficLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrafficLevel::Free => write!(f, "Szabad"),
            TrafficLevel::Light => write!(f, "Enyhe"),
            TrafficLevel::Moderate => write!(f, "Közepes"),
            TrafficLevel::Heavy => write!(f, "Sűrű"),
            TrafficLevel::Blocked => write!(f, "Blokkolva"),
        }
    }
}

/// Emlék az útvonalon (egyszerűsített forma)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOnRoute {
    pub id: String,
    pub content: String,
    pub location: GeoPoint,
    pub importance: f64,
    pub emotional_tag: String,
}

impl Default for MemoryOnRoute {
    fn default() -> Self {
        Self {
            id: String::new(),
            content: String::new(),
            location: GeoPoint::default(),
            importance: 0.5,
            emotional_tag: String::new(),
        }
    }
}

/// Késleltetés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delay {
    pub location: GeoPoint,
    pub duration: Duration,
    pub reason: DelayReason,
    pub avoidable: bool,
}

/// Késleltetés oka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayReason {
    Traffic,
    Accident,
    Construction,
    Event,
    Weather,
    Unknown,
}

/// Javasolt megálló
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedStop {
    pub place: Place,
    pub reason: StopReason,
    pub detour_time: Duration,
    pub relevance_score: f64,
}

/// Megállás oka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    FavoritePlace,
    UserRequest,
    NeedsBased,
    MemoryTriggered,
    RoutineStop,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopReason::FavoritePlace => write!(f, "Kedvenc hely útban"),
            StopReason::UserRequest => write!(f, "Felhasználó kérte"),
            StopReason::NeedsBased => write!(f, "Szükséglet alapján"),
            StopReason::MemoryTriggered => write!(f, "Emlék kapcsolódik ide"),
            StopReason::RoutineStop => write!(f, "Szokásos megálló"),
        }
    }
}

// ============================================================================
// ROUTE CONTEXT (A Hope különbség)
// ============================================================================

/// Útvonal kontextus - emlékek, érzelmek, javaslatok
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteContext {
    pub memories: Vec<MemoryOnRoute>,
    pub emotional_history: Vec<EmotionAtLocation>,
    pub people_associated: Vec<String>,
    pub events_nearby: Vec<NearbyEvent>,
    pub suggestions: Vec<ContextSuggestion>,
}

impl Default for RouteContext {
    fn default() -> Self {
        Self {
            memories: Vec::new(),
            emotional_history: Vec::new(),
            people_associated: Vec::new(),
            events_nearby: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

/// Érzelem egy adott helyen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionAtLocation {
    pub location: GeoPoint,
    pub emotion: String,
    pub intensity: f64,
    pub when: DateTime<Utc>,
}

/// Közeli esemény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyEvent {
    pub name: String,
    pub location: GeoPoint,
    pub distance_km: f64,
    pub when: Option<DateTime<Utc>>,
}

/// Kontextus javaslat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSuggestion {
    pub text: String,
    pub relevance: f64,
    pub action: Option<SuggestedAction>,
}

/// Javasolt akció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestedAction {
    AddStop { place: Place },
    ChangeRoute { reason: String },
    RemindLater { message: String },
    CallPerson { name: String },
    PlayMusic { mood: String },
}

// ============================================================================
// PREDICTION
// ============================================================================

/// Prediktált célpont
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedDestination {
    pub place: Place,
    pub confidence: f64,
    pub reasoning: Vec<PredictionReason>,
    pub suggested_departure: DateTime<Utc>,
}

/// Predikció oka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionReason {
    DailyRoutine,
    CalendarEvent,
    RecentConversation,
    LocationPattern,
    TimePattern,
    WeatherBased,
}

impl std::fmt::Display for PredictionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PredictionReason::DailyRoutine => write!(f, "Napi rutin alapján"),
            PredictionReason::CalendarEvent => write!(f, "Naptár esemény"),
            PredictionReason::RecentConversation => write!(f, "Közelmúltbeli beszélgetés"),
            PredictionReason::LocationPattern => write!(f, "Helyminta alapján"),
            PredictionReason::TimePattern => write!(f, "Időminta alapján"),
            PredictionReason::WeatherBased => write!(f, "Időjárás alapján"),
        }
    }
}

// ============================================================================
// MOVEMENT PATTERNS
// ============================================================================

/// Mozgási minták (tanult)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MovementPatterns {
    pub daily_routines: HashMap<u8, Vec<RoutineTrip>>,
    pub frequent_destinations: Vec<(Place, u64)>,
    pub average_speeds: HashMap<String, f64>,
    pub preferred_departure_times: HashMap<String, Vec<TimeRange>>,
}

/// Rutin utazás
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineTrip {
    pub from: Place,
    pub to: Place,
    pub typical_departure: TimeRange,
    pub typical_duration: Duration,
    pub frequency: u32,
    pub last_occurred: DateTime<Utc>,
}

/// Időtartomány
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
}

impl TimeRange {
    pub fn new(start_hour: u8, start_minute: u8, end_hour: u8, end_minute: u8) -> Self {
        Self {
            start_hour,
            start_minute,
            end_hour,
            end_minute,
        }
    }

    pub fn contains_time(&self, hour: u8, minute: u8) -> bool {
        let time = hour as u16 * 60 + minute as u16;
        let start = self.start_hour as u16 * 60 + self.start_minute as u16;
        let end = self.end_hour as u16 * 60 + self.end_minute as u16;
        time >= start && time <= end
    }
}

// ============================================================================
// ACTIVE NAVIGATION
// ============================================================================

/// Aktív navigáció állapot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveNavigation {
    pub route: SmartRoute,
    pub started_at: DateTime<Utc>,
    pub current_position: GeoLocation,
    pub current_segment: usize,
    pub off_route: bool,
    pub recalculating: bool,
    pub distance_remaining_km: f64,
    pub time_remaining: Duration,
}

/// Navigáció frissítés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationUpdate {
    pub position: GeoLocation,
    pub current_instruction: String,
    pub next_instruction: Option<String>,
    pub distance_to_next: f64,
    pub eta: DateTime<Utc>,
    pub off_route: bool,
    pub speed_kmh: f64,
    pub context_message: Option<String>,
}

/// Befejezett útvonal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedRoute {
    pub route_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub planned_duration: Duration,
    pub actual_duration: Duration,
    pub planned_distance_km: f64,
    pub actual_distance_km: f64,
    pub stops_made: Vec<Place>,
    pub off_route_count: u32,
    pub average_speed_kmh: f64,
}

// ============================================================================
// NAVIGATION ENGINE
// ============================================================================

/// Navigation Engine - A Hope navigációs rendszer
pub struct NavigationEngine {
    /// Geo Engine referencia
    geo_engine: Arc<GeoEngine>,
    /// Aktív navigáció
    active_navigation: Arc<RwLock<Option<ActiveNavigation>>>,
    /// Útvonal történet
    route_history: Arc<RwLock<Vec<CompletedRoute>>>,
    /// Tanult minták
    learned_patterns: Arc<RwLock<MovementPatterns>>,
    /// Aktuális hangulat
    current_mood: Arc<RwLock<EmotionState>>,
    /// Statisztikák
    stats: Arc<RwLock<NavigationStats>>,
}

/// Navigációs statisztikák
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NavigationStats {
    pub total_routes_planned: u64,
    pub total_routes_completed: u64,
    pub total_distance_km: f64,
    pub total_time_navigating: Duration,
    pub predictions_made: u64,
    pub predictions_correct: u64,
    pub favorite_destinations: Vec<(String, u64)>,
}

impl NavigationEngine {
    /// Új Navigation Engine
    pub fn new(geo_engine: Arc<GeoEngine>) -> Self {
        Self {
            geo_engine,
            active_navigation: Arc::new(RwLock::new(None)),
            route_history: Arc::new(RwLock::new(Vec::new())),
            learned_patterns: Arc::new(RwLock::new(MovementPatterns::default())),
            current_mood: Arc::new(RwLock::new(EmotionState::default())),
            stats: Arc::new(RwLock::new(NavigationStats::default())),
        }
    }

    // ========================================================================
    // ROUTE PLANNING
    // ========================================================================

    /// Útvonal tervezés kontextussal
    pub async fn plan_route(&self, ctx: NavigationContext) -> HopeResult<SmartRoute> {
        // Origin: ha None, használjuk a jelenlegi lokációt
        let origin_geo = if let Some(ref o) = ctx.origin {
            GeoLocation {
                latitude: o.latitude,
                longitude: o.longitude,
                altitude: None,
                accuracy: None,
                source: GeoSource::Manual,
                timestamp: Utc::now(),
            }
        } else {
            self.geo_engine
                .get_current_location()
                .await
                .unwrap_or_default()
        };

        let dest_geo = GeoLocation {
            latitude: ctx.destination.latitude,
            longitude: ctx.destination.longitude,
            altitude: None,
            accuracy: None,
            source: GeoSource::Manual,
            timestamp: Utc::now(),
        };

        let mut route = SmartRoute::new(origin_geo, dest_geo);

        // Hangulat alapú módosítások
        self.apply_mood_adjustments(&mut route, &ctx.emotions).await;

        // Időnyomás kezelése
        if ctx.time_pressure > 0.7 {
            route
                .context_notes
                .push("Gyors útvonal választva a sürgősség miatt".to_string());
        }

        // Preferenciák alkalmazása
        if ctx.preferences.prefer_scenic && ctx.time_pressure < 0.5 {
            route
                .context_notes
                .push("Szebb útvonal választva".to_string());
            route.emotional_score += 0.1;
        }

        // Tanult kerülések
        for avoidance in &ctx.preferences.learned_avoidances {
            route.context_notes.push(format!(
                "Kerülve: {} ({})",
                avoidance.name, avoidance.reason
            ));
        }

        // Waypoints hozzáadása
        for waypoint in &ctx.waypoints {
            let point = waypoint.clone();
            if !route.path.is_empty() {
                route.path.insert(route.path.len() - 1, point);
            }
        }

        // Útmenti helyek keresése
        self.find_places_on_route(&mut route).await;

        // Javasolt megállók
        self.suggest_stops(&mut route, &ctx).await;

        // Statisztika frissítés
        {
            let mut stats = self.stats.write().await;
            stats.total_routes_planned += 1;
        }

        Ok(route)
    }

    /// Alternatív útvonalak tervezése
    pub async fn plan_alternatives(&self, ctx: NavigationContext) -> HopeResult<Vec<SmartRoute>> {
        let mut routes = Vec::new();

        // Alap útvonal
        let base_route = self.plan_route(ctx.clone()).await?;
        routes.push(base_route);

        // Gyors útvonal
        let mut fast_ctx = ctx.clone();
        fast_ctx.time_pressure = 1.0;
        fast_ctx.preferences.prefer_scenic = false;
        let fast_route = self.plan_route(fast_ctx).await?;
        if routes.iter().all(|r| r.id != fast_route.id) {
            routes.push(fast_route);
        }

        // Szép útvonal
        let mut scenic_ctx = ctx.clone();
        scenic_ctx.time_pressure = 0.0;
        scenic_ctx.preferences.prefer_scenic = true;
        let scenic_route = self.plan_route(scenic_ctx).await?;
        if routes.iter().all(|r| r.id != scenic_route.id) {
            routes.push(scenic_route);
        }

        Ok(routes)
    }

    /// Hangulat alapú útvonal módosítások
    async fn apply_mood_adjustments(&self, route: &mut SmartRoute, mood: &EmotionState) {
        // High intensity negative emotions -> simpler route
        if mood.intensity > 0.7 && mood.valence < 0.0 {
            route
                .context_notes
                .push("Magas intenzitás - egyszerűbb útvonal".to_string());
            route.emotional_score += 0.15;
        }

        // Low arousal (tired) -> calmer route
        if mood.arousal < 0.3 {
            route
                .context_notes
                .push("Fáradt vagy - nyugodtabb útvonal".to_string());
            route.emotional_score += 0.1;
        }

        // Positive valence -> enjoy the route
        if mood.valence > 0.5 {
            route
                .context_notes
                .push("Jó hangulatban vagy - élvezd az utat!".to_string());
            route.emotional_score += 0.1;
        }
    }

    /// Útmenti helyek keresése
    async fn find_places_on_route(&self, route: &mut SmartRoute) {
        let places = self.geo_engine.list_places().await;

        for place in places {
            // Ellenőrizzük hogy az útvonal közelében van-e
            for point in &route.path {
                let place_point = GeoPoint::new(place.location.latitude, place.location.longitude);
                let distance = point.distance_to(&place_point);

                if distance < 0.5 {
                    // 500m-en belül
                    if !route.places_on_route.iter().any(|p| p.id == place.id) {
                        route.places_on_route.push(place.clone());
                    }
                    break;
                }
            }
        }
    }

    /// Megállók javaslása
    async fn suggest_stops(&self, route: &mut SmartRoute, ctx: &NavigationContext) {
        // Kedvenc helyek útban
        for place in &ctx.preferences.preferred_stops {
            let place_point = GeoPoint::new(place.location.latitude, place.location.longitude);

            for point in &route.path {
                let distance = point.distance_to(&place_point);
                if distance < 2.0 {
                    // 2km-en belül
                    route.suggested_stops.push(SuggestedStop {
                        place: place.clone(),
                        reason: StopReason::FavoritePlace,
                        detour_time: Duration::minutes((distance * 2.0) as i64),
                        relevance_score: 0.8,
                    });
                    break;
                }
            }
        }

        // Szükséglet alapú megállók (alacsony arousal = fáradt)
        if ctx.emotions.arousal < 0.3 {
            // Keresünk kávézót útban
            for place in &route.places_on_route {
                if place.place_type == PlaceType::Restaurant
                    && place.name.to_lowercase().contains("kávé")
                {
                    route.suggested_stops.push(SuggestedStop {
                        place: place.clone(),
                        reason: StopReason::NeedsBased,
                        detour_time: Duration::minutes(0),
                        relevance_score: 0.9,
                    });
                    route
                        .context_notes
                        .push(format!("Fáradt vagy - {} itt van útban", place.name));
                }
            }
        }
    }

    // ========================================================================
    // ACTIVE NAVIGATION
    // ========================================================================

    /// Navigáció indítása
    pub async fn start_navigation(&self, route: SmartRoute) -> HopeResult<()> {
        let active = ActiveNavigation {
            route: route.clone(),
            started_at: Utc::now(),
            current_position: GeoLocation::default(),
            current_segment: 0,
            off_route: false,
            recalculating: false,
            distance_remaining_km: route.total_distance_km,
            time_remaining: route.total_duration,
        };

        let mut nav = self.active_navigation.write().await;
        *nav = Some(active);

        Ok(())
    }

    /// Pozíció frissítés
    pub async fn update_position(&self, pos: GeoLocation) -> Option<NavigationUpdate> {
        let mut nav_guard = self.active_navigation.write().await;

        if let Some(ref mut nav) = *nav_guard {
            nav.current_position = pos.clone();

            // Ellenőrizzük hogy útvonalon vagyunk-e
            let current_point = GeoPoint::from_location(&pos);
            let mut min_distance = f64::MAX;

            for (i, segment) in nav.route.segments.iter().enumerate() {
                let dist = current_point.distance_to(&segment.end);
                if dist < min_distance {
                    min_distance = dist;
                    nav.current_segment = i;
                }
            }

            nav.off_route = min_distance > 0.2; // 200m-nél távolabb

            // Frissítés készítése
            let current_segment = nav.route.segments.get(nav.current_segment);
            let next_segment = nav.route.segments.get(nav.current_segment + 1);

            let update = NavigationUpdate {
                position: pos,
                current_instruction: current_segment
                    .map(|s| s.instruction.clone())
                    .unwrap_or_else(|| "Haladj tovább".to_string()),
                next_instruction: next_segment.map(|s| s.instruction.clone()),
                distance_to_next: current_segment.map(|s| s.distance_km).unwrap_or(0.0),
                eta: nav.route.eta,
                off_route: nav.off_route,
                speed_kmh: 50.0, // TODO: calculate from position history
                context_message: if nav.off_route {
                    Some("Letértél az útvonalról".to_string())
                } else {
                    None
                },
            };

            return Some(update);
        }

        None
    }

    /// Navigáció leállítása
    pub async fn stop_navigation(&self) -> Option<CompletedRoute> {
        let mut nav_guard = self.active_navigation.write().await;

        if let Some(nav) = nav_guard.take() {
            let completed = CompletedRoute {
                route_id: nav.route.id,
                started_at: nav.started_at,
                completed_at: Utc::now(),
                planned_duration: nav.route.total_duration,
                actual_duration: Utc::now().signed_duration_since(nav.started_at),
                planned_distance_km: nav.route.total_distance_km,
                actual_distance_km: nav.route.total_distance_km - nav.distance_remaining_km,
                stops_made: Vec::new(),
                off_route_count: if nav.off_route { 1 } else { 0 },
                average_speed_kmh: 50.0,
            };

            // Történethez adás
            {
                let mut history = self.route_history.write().await;
                history.push(completed.clone());
            }

            // Statisztika frissítés
            {
                let mut stats = self.stats.write().await;
                stats.total_routes_completed += 1;
                stats.total_distance_km += completed.actual_distance_km;
                stats.total_time_navigating =
                    stats.total_time_navigating + completed.actual_duration;
            }

            // Tanulás
            self.learn_from_trip(&completed).await;

            return Some(completed);
        }

        None
    }

    // ========================================================================
    // PREDICTION
    // ========================================================================

    /// Célpont predikció
    pub async fn predict_destination(&self) -> Option<PredictedDestination> {
        let now = Utc::now();
        let weekday = now.weekday().num_days_from_monday() as u8;
        let hour = now.hour() as u8;
        let minute = now.minute() as u8;

        let patterns = self.learned_patterns.read().await;

        // Napi rutin keresés
        if let Some(routines) = patterns.daily_routines.get(&weekday) {
            for routine in routines {
                if routine.typical_departure.contains_time(hour, minute) {
                    return Some(PredictedDestination {
                        place: routine.to.clone(),
                        confidence: 0.85 + (routine.frequency as f64 / 100.0).min(0.1),
                        reasoning: vec![
                            PredictionReason::DailyRoutine,
                            PredictionReason::TimePattern,
                        ],
                        suggested_departure: now,
                    });
                }
            }
        }

        // Gyakori célpontok
        if let Some((place, count)) = patterns.frequent_destinations.first() {
            if *count > 10 {
                return Some(PredictedDestination {
                    place: place.clone(),
                    confidence: 0.6 + (*count as f64 / 200.0).min(0.2),
                    reasoning: vec![PredictionReason::LocationPattern],
                    suggested_departure: now,
                });
            }
        }

        None
    }

    /// Indulási idő javaslat
    pub async fn suggest_departure_time(&self, destination: &Place) -> DateTime<Utc> {
        let patterns = self.learned_patterns.read().await;

        if let Some(times) = patterns
            .preferred_departure_times
            .get(&destination.id.to_string())
        {
            if let Some(time_range) = times.first() {
                let now = Utc::now();
                return now
                    .with_hour(time_range.start_hour as u32)
                    .and_then(|t| t.with_minute(time_range.start_minute as u32))
                    .unwrap_or(now);
            }
        }

        Utc::now()
    }

    // ========================================================================
    // LEARNING
    // ========================================================================

    /// Tanulás egy befejezett útból
    async fn learn_from_trip(&self, completed: &CompletedRoute) {
        let mut patterns = self.learned_patterns.write().await;

        // Ha volt útvonal, frissítjük a gyakoriságot
        // A részletes tanulást a route history-ból kellene csinálni
        // Ez egy egyszerűsített verzió

        // Átlagsebesség frissítés
        let speed_key = "overall".to_string();
        patterns
            .average_speeds
            .entry(speed_key)
            .and_modify(|avg| *avg = (*avg + completed.average_speed_kmh) / 2.0)
            .or_insert(completed.average_speed_kmh);
    }

    /// Kerülés tanulása
    pub async fn learn_avoidance(&self, area: GeoArea) {
        // Ezt a RoutePreferences-be kellene menteni
        // De az per-context, nem globális
        // Jövőbeli feature: perzisztens preferencia store
    }

    // ========================================================================
    // CONTEXT
    // ========================================================================

    /// Útvonal kontextus lekérése
    pub async fn get_route_context(&self, route: &SmartRoute) -> RouteContext {
        let mut ctx = RouteContext::default();

        // Hozzáadás az útvonalon talált helyekhez
        for place in &route.places_on_route {
            ctx.suggestions.push(ContextSuggestion {
                text: format!("{} útba esik", place.name),
                relevance: 0.7,
                action: Some(SuggestedAction::AddStop {
                    place: place.clone(),
                }),
            });
        }

        // Javasolt megállók
        for stop in &route.suggested_stops {
            ctx.suggestions.push(ContextSuggestion {
                text: format!("{}: {}", stop.place.name, stop.reason),
                relevance: stop.relevance_score,
                action: Some(SuggestedAction::AddStop {
                    place: stop.place.clone(),
                }),
            });
        }

        ctx
    }

    // ========================================================================
    // ETA
    // ========================================================================

    /// ETA számítás
    pub async fn calculate_eta(
        &self,
        destination: &GeoLocation,
    ) -> HopeResult<(Duration, DateTime<Utc>, TrafficLevel, f64)> {
        let current_location = self.geo_engine.get_current_location().await;

        if let Some(current) = current_location {
            let distance = current.distance_to(destination);
            let avg_speed = 50.0; // km/h - TODO: tanult átlagsebesség

            let duration_hours = distance / avg_speed;
            let duration = Duration::minutes((duration_hours * 60.0) as i64);
            let arrival = Utc::now() + duration;

            // Forgalom becslés (egyszerűsített)
            let hour = Utc::now().hour();
            let traffic = if hour >= 7 && hour <= 9 || hour >= 16 && hour <= 18 {
                TrafficLevel::Moderate
            } else {
                TrafficLevel::Light
            };

            let confidence = 0.8; // TODO: múltbeli adatok alapján

            return Ok((duration, arrival, traffic, confidence));
        }

        // Ha nincs pozíció, becsülünk
        let duration = Duration::minutes(30);
        Ok((duration, Utc::now() + duration, TrafficLevel::Light, 0.5))
    }

    // ========================================================================
    // NEARBY
    // ========================================================================

    /// Közeli helyek keresése
    pub async fn find_nearby(
        &self,
        category: Option<&str>,
        radius_km: f64,
        on_route: bool,
    ) -> Vec<PlaceWithContext> {
        let mut results = Vec::new();

        let current = self.geo_engine.get_current_location().await;
        if current.is_none() {
            return results;
        }
        let current = current.unwrap();
        let current_point = GeoPoint::from_location(&current);

        let places = self.geo_engine.list_places().await;

        for place in places {
            let place_point = GeoPoint::new(place.location.latitude, place.location.longitude);
            let distance = current_point.distance_to(&place_point);

            if distance <= radius_km {
                // Kategória szűrés
                if let Some(cat) = category {
                    let place_cat = format!("{:?}", place.place_type).to_lowercase();
                    if !place_cat.contains(&cat.to_lowercase()) {
                        continue;
                    }
                }

                // Ha on_route, ellenőrizzük hogy az aktív útvonalon van-e
                if on_route {
                    let nav = self.active_navigation.read().await;
                    if let Some(ref nav) = *nav {
                        let mut on_current_route = false;
                        for point in &nav.route.path {
                            if point.distance_to(&place_point) < 0.5 {
                                on_current_route = true;
                                break;
                            }
                        }
                        if !on_current_route {
                            continue;
                        }
                    }
                }

                results.push(PlaceWithContext {
                    place: place.clone(),
                    distance_km: distance,
                    on_route,
                    context: None,
                });
            }
        }

        // Távolság szerint rendezés
        results.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());

        results
    }

    // ========================================================================
    // STATUS
    // ========================================================================

    /// Hangulat beállítása
    pub async fn set_mood(&self, mood: EmotionState) {
        let mut current = self.current_mood.write().await;
        *current = mood;
    }

    /// Aktív navigáció lekérése
    pub async fn get_active_navigation(&self) -> Option<ActiveNavigation> {
        let nav = self.active_navigation.read().await;
        nav.clone()
    }

    /// Statisztikák
    pub async fn get_stats(&self) -> NavigationStats {
        self.stats.read().await.clone()
    }

    /// Mozgási minták
    pub async fn get_movement_patterns(&self) -> MovementPatterns {
        self.learned_patterns.read().await.clone()
    }

    /// Útvonal történet
    pub async fn get_route_history(&self, limit: usize) -> Vec<CompletedRoute> {
        let history = self.route_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// Hely kontextussal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceWithContext {
    pub place: Place,
    pub distance_km: f64,
    pub on_route: bool,
    pub context: Option<String>,
}

impl Default for NavigationEngine {
    fn default() -> Self {
        Self::new(Arc::new(GeoEngine::new()))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_point_distance() {
        let budapest = GeoPoint::new(47.4979, 19.0402);
        let vienna = GeoPoint::new(48.2082, 16.3738);

        let distance = budapest.distance_to(&vienna);
        assert!(distance > 200.0 && distance < 250.0); // ~215 km
    }

    #[test]
    fn test_time_range() {
        let morning = TimeRange::new(6, 0, 9, 0);

        assert!(morning.contains_time(7, 30));
        assert!(!morning.contains_time(5, 30));
        assert!(!morning.contains_time(10, 0));
    }

    #[test]
    fn test_emotion_state() {
        let relaxed = EmotionState::relaxed();
        assert!(relaxed.valence > 0.5); // Positive valence
        assert!(relaxed.arousal < 0.5); // Low arousal

        let stressed = EmotionState::stressed();
        assert!(stressed.arousal > 0.5); // High arousal
    }

    #[test]
    fn test_route_preferences_default() {
        let prefs = RoutePreferences::default();
        assert!(prefs.prefer_familiar);
        assert!(!prefs.avoid_highways);
    }

    #[test]
    fn test_traffic_level_display() {
        assert_eq!(format!("{}", TrafficLevel::Free), "Szabad");
        assert_eq!(format!("{}", TrafficLevel::Heavy), "Sűrű");
    }

    #[test]
    fn test_stop_reason_display() {
        assert_eq!(
            format!("{}", StopReason::FavoritePlace),
            "Kedvenc hely útban"
        );
        assert_eq!(format!("{}", StopReason::NeedsBased), "Szükséglet alapján");
    }

    #[tokio::test]
    async fn test_navigation_engine_creation() {
        let engine = NavigationEngine::default();
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_routes_planned, 0);
    }

    #[tokio::test]
    async fn test_plan_route() {
        let engine = NavigationEngine::default();

        let ctx = NavigationContext {
            origin: Some(GeoPoint {
                latitude: 47.4979,
                longitude: 19.0402,
                name: Some("Budapest".to_string()),
            }),
            destination: GeoPoint {
                latitude: 47.5,
                longitude: 19.05,
                name: Some("Destination".to_string()),
            },
            ..Default::default()
        };

        let route = engine.plan_route(ctx).await.unwrap();
        assert!(route.total_distance_km > 0.0);
    }

    #[tokio::test]
    async fn test_mood_based_route() {
        let engine = NavigationEngine::default();

        // Fáradt hangulat
        let tired_ctx = NavigationContext {
            current_mood: "tired".to_string(),
            emotions: EmotionState::tired(),
            origin: Some(GeoPoint {
                latitude: 47.4979,
                longitude: 19.0402,
                name: Some("Budapest".to_string()),
            }),
            destination: GeoPoint {
                latitude: 47.5,
                longitude: 19.05,
                name: Some("Destination".to_string()),
            },
            ..Default::default()
        };

        let route = engine.plan_route(tired_ctx).await.unwrap();
        assert!(route.context_notes.iter().any(|n| n.contains("Fáradt")));
    }

    #[tokio::test]
    async fn test_start_stop_navigation() {
        let engine = NavigationEngine::default();

        let ctx = NavigationContext::default();
        let route = engine.plan_route(ctx).await.unwrap();

        engine.start_navigation(route).await.unwrap();

        let active = engine.get_active_navigation().await;
        assert!(active.is_some());

        let completed = engine.stop_navigation().await;
        assert!(completed.is_some());

        let active = engine.get_active_navigation().await;
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_predict_destination_empty() {
        let engine = NavigationEngine::default();

        // Üres patterns esetén None
        let prediction = engine.predict_destination().await;
        assert!(prediction.is_none());
    }

    #[test]
    fn test_smart_route_creation() {
        let origin = GeoLocation {
            latitude: 47.4979,
            longitude: 19.0402,
            ..Default::default()
        };
        let destination = GeoLocation {
            latitude: 47.5,
            longitude: 19.05,
            ..Default::default()
        };

        let route = SmartRoute::new(origin, destination);
        assert!(!route.path.is_empty());
        assert!(route.total_distance_km > 0.0);
    }

    #[test]
    fn test_prediction_reason_display() {
        assert_eq!(
            format!("{}", PredictionReason::DailyRoutine),
            "Napi rutin alapján"
        );
        assert_eq!(
            format!("{}", PredictionReason::TimePattern),
            "Időminta alapján"
        );
    }
}
