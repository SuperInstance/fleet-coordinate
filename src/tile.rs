//! PLATO Tile Integration — fleet coordination via tile forwarding

use serde::{Deserialize, Serialize};

/// TileBounds — encodes Laman threshold into tile geometry.
/// Inspired by FM's HolonomyBounds in holonomy-consensus/src/constraints.rs.
/// A tile that exceeds its deviation bound is a constraint violation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileBounds {
    /// Maximum deviation from equilibrium position
    pub max_deviation: f64,
    /// Cycles before rechecking
    pub max_age: u32,
    /// Minimum neighbors that must agree (Laman threshold: 2v'-3 per subgraph)
    pub min_agreement: usize,
}

impl TileBounds {
    pub fn new(max_deviation: f64, max_age: u32, min_agreement: usize) -> Self {
        Self { max_deviation, max_age, min_agreement }
    }

    /// Check if a given deviation is within bounds
    pub fn check_bounds(&self, deviation: f64) -> bool {
        deviation <= self.max_deviation
    }

    /// Default bounds for fleet tiles
    pub fn default_fleet() -> Self {
        Self {
            max_deviation: 1.0,
            max_age: 100,
            min_agreement: 3, // minimum agents for Laman-rigid subgraph
        }
    }
}

/// A PLATO tile forwarded through the fleet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetTile {
    pub room: String,
    pub question: String,
    pub answer: String,
    pub domain: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub timestamp: f64,
}

impl FleetTile {
    pub fn new(room: String, question: String, answer: String) -> Self {
        Self {
            room,
            question,
            answer,
            domain: None,
            tags: vec![],
            author: "fleet".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as f64)
                .unwrap_or(0.0),
        }
    }

    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Tile coordination state for a room
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileCoordination {
    pub room: String,
    pub tiles: Vec<FleetTile>,
    pub consensus_tiles: Vec<String>,
    pub pending_tiles: Vec<String>,
}

impl TileCoordination {
    pub fn new(room: String) -> Self {
        Self {
            room,
            tiles: Vec::new(),
            consensus_tiles: Vec::new(),
            pending_tiles: Vec::new(),
        }
    }

    pub fn add_tile(&mut self, tile: FleetTile) {
        if !self.tiles.iter().any(|t| t.question == tile.question) {
            self.tiles.push(tile.clone());
            self.pending_tiles.push(tile.question.clone());
        }
    }

    pub fn mark_consensus(&mut self, question: &str) {
        self.pending_tiles.retain(|q| q != question);
        if !self.consensus_tiles.contains(&question.to_string()) {
            self.consensus_tiles.push(question.to_string());
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }
}

/// Integration with cocapn-glue-core TILE messages
pub fn tile_to_glue(tile: &FleetTile) -> Vec<u8> {
    let msg = serde_json::json!({
        "t": 0x05,
        "room": tile.room,
        "question": tile.question,
        "answer": tile.answer,
        "author": tile.author,
    });
    serde_json::to_vec(&msg).unwrap_or_default()
}

/// Parse a glue TILE message into a FleetTile
pub fn glue_to_tile(data: &[u8]) -> Option<FleetTile> {
    let msg: serde_json::Value = serde_json::from_slice(data).ok()?;
    Some(FleetTile {
        room: msg["room"].as_str()?.to_string(),
        question: msg["question"].as_str()?.to_string(),
        answer: msg["answer"].as_str()?.to_string(),
        domain: msg["domain"].as_str().map(String::from),
        tags: msg["tags"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        author: msg["author"].as_str().unwrap_or("fleet").to_string(),
        timestamp: msg["timestamp"].as_f64().unwrap_or(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_roundtrip() {
        let tile = FleetTile::new(
            "fleet_math".to_string(),
            "What is the Betti number beta1?".to_string(),
            "beta1 = E - V + C for a graph with C connected components.".to_string(),
        );
        let data = tile_to_glue(&tile);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_coordination_mark_consensus() {
        let mut coord = TileCoordination::new("test".to_string());
        coord.add_tile(FleetTile::new("test".to_string(), "Q1?".to_string(), "A1".to_string()));
        coord.add_tile(FleetTile::new("test".to_string(), "Q2?".to_string(), "A2".to_string()));
        assert_eq!(coord.tile_count(), 2);
        assert_eq!(coord.pending_tiles.len(), 2);
        coord.mark_consensus("Q1?");
        assert_eq!(coord.consensus_tiles.len(), 1);
        assert_eq!(coord.pending_tiles.len(), 1);
    }

    #[test]
    fn test_tilebounds_check_bounds() {
        let bounds = TileBounds::new(1.0, 100, 3);
        assert!(bounds.check_bounds(0.5));
        assert!(bounds.check_bounds(1.0));
        assert!(!bounds.check_bounds(1.1));
    }


    #[test]
    fn test_tilebounds_default_fleet() {
        let bounds = TileBounds::default_fleet();
        assert!(bounds.check_bounds(0.5));
        assert_eq!(bounds.max_age, 100);
        assert_eq!(bounds.min_agreement, 3);
    }

    #[test]
    fn test_tilebounds_zero_deviation() {
        let bounds = TileBounds::new(0.0, 50, 5);
        assert!(bounds.check_bounds(0.0));
        assert!(!bounds.check_bounds(0.001));
    }

    #[test]
    fn test_tilebounds_exactly_at_bound() {
        let bounds = TileBounds::new(2.5, 200, 4);
        assert!(bounds.check_bounds(2.5));
        assert!(!bounds.check_bounds(2.5001));
    }
}

// === Zeroclaw Hibernation Integration (2026-05-06) ===
// See: /home/ubuntu/.openclaw/workspace/research/zeroclaw-hibernation-synthesis.md

/// Tile hibernation states — prevents wasted consensus checks on stale tiles
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum TileHibernationState {
    #[default]
    Active,
    Hibernating,
    WakePending,
}

/// Check if a tile should enter hibernation based on idle time
/// 
/// Triggers hibernation after:
/// - 30 minutes of no new observations (from zeroclaw "Slumber" protocol)
/// - Energy threshold: energy budget below 20%
/// - Queue depth below 5 pending updates
pub fn check_hibernation_trigger(
    last_activity_timestamp: u64,
    current_time_ms: u64,
    energy_budget_pct: f64,
    pending_queue_depth: usize,
) -> bool {
    let idle_ms = current_time_ms.saturating_sub(last_activity_timestamp);
    let idle_threshold_ms = 30 * 60 * 1000; // 30 minutes
    
    idle_ms > idle_threshold_ms 
        || energy_budget_pct < 0.20 
        || pending_queue_depth < 5
}

/// Returns DVFS power level for tile consensus work
/// Based on zeroclaw "Slumber" DVFS settings
pub fn dvfs_power_level(is_idle: bool) -> (f64, f64) {
    // (voltage_v, frequency_mhz)
    if is_idle {
        (0.6, 100.0)  // 0.25 mW — 75% energy reduction
    } else {
        (1.2, 2500.0) // 10 mW — full operation
    }
}
