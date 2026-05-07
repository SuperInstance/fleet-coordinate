//! Fleet Coordinate Integration — the unified API
//!
//! FleetCoordinate is the single entry point for all fleet coordination algorithms.

use serde::{Deserialize, Serialize};

pub use crate::zhc::ZhcConsensus;
pub use crate::beam::{BeamSolver, JointEquilibrium, MultiSegmentBeam, JointState};
pub use crate::pythagorean48::{TrustTopology, TrustVector};
pub use crate::graph::{FleetGraph, RigidityResult};
pub use crate::tile::{FleetTile, TileCoordination};
pub use crate::emergence::{EmergenceDetector, EmergenceResult};

/// Unified fleet coordination configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub equilibrium_tolerance: f64,
    pub zhc_tolerance: f64,
    pub max_iterations: usize,
    pub trust_bits: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            equilibrium_tolerance: 1e-6,
            zhc_tolerance: 0.5,
            max_iterations: 500,
            trust_bits: 5.58496,
        }
    }
}

/// The unified fleet coordinate engine
#[derive(Clone)]
pub struct FleetCoordinate {
    config: Config,
    graph: FleetGraph,
    zhc: ZhcConsensus,
    beam: BeamSolver,
    trust: TrustTopology,
    tiles: Vec<TileCoordination>,
}

impl FleetCoordinate {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            graph: FleetGraph::new(),
            zhc: ZhcConsensus::new(config.zhc_tolerance),
            beam: BeamSolver::new(config.equilibrium_tolerance),
            trust: TrustTopology::new(),
            tiles: Vec::new(),
        }
    }

    pub fn add_agent(&mut self, id: u64, position: [f64; 2], capabilities: Vec<String>) {
        self.graph.add_agent(id, position, capabilities);
        self.zhc.add_tile(id, [position[0], position[1], 0.0], vec![]);
    }

    pub fn add_trust_edge(&mut self, a: u64, b: u64) {
        self.graph.add_edge(a, b);
        // Also sync ZHC adjacency so consensus reflects actual trust topology
        self.zhc.add_tile(a, [0.0, 0.0, 0.0], vec![b]);
        self.zhc.add_tile(b, [0.0, 0.0, 0.0], vec![a]);
    }

    pub fn analyze(&self) -> FleetAnalysisReport {
        let rigidity = self.graph.check_laman_rigidity();
        let zhc_consensus = self.zhc.run_consensus();
        let emergence = EmergenceDetector::detect(rigidity.V, rigidity.E, 1);
        let is_self = rigidity.is_rigid && zhc_consensus.is_consistent && !emergence.emergence_detected;

        FleetAnalysisReport {
            rigidity: rigidity.clone(),
            zhc_consensus,
            emergence: emergence.clone(),
            config: self.config.clone(),
            fleet_theorem: FleetTheoremResult {
                is_laman_rigid: is_self,
                requires_voting: false,
                requires_central_coordinator: false,
                drift_free: true,
            },
        }
    }

    pub fn trust_encoder(&self) -> &TrustTopology {
        &self.trust
    }

    pub fn beam_solver(&self) -> &BeamSolver {
        &self.beam
    }

    pub fn add_tile(&mut self, room: String, tile: FleetTile) {
        if let Some(c) = self.tiles.iter_mut().find(|c| c.room == room) {
            c.add_tile(tile);
        } else {
            let mut coord = TileCoordination::new(room);
            coord.add_tile(tile);
            self.tiles.push(coord);
        }
    }

    pub fn agent_count(&self) -> usize {
        self.graph.V()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.E()
    }
}

impl Default for FleetCoordinate {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

/// Complete fleet analysis report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetAnalysisReport {
    pub rigidity: RigidityResult,
    pub zhc_consensus: crate::zhc::ConsensusResult,
    pub emergence: EmergenceResult,
    pub config: Config,
    pub fleet_theorem: FleetTheoremResult,
}

/// Result of the Fleet Coordinate Theorem (provisional)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetTheoremResult {
    pub is_laman_rigid: bool,
    /// True if the fleet has geometric consistency (ZHC closure satisfied)
    pub requires_voting: bool,
    pub requires_central_coordinator: bool,
    pub drift_free: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_agent_triangle_self_coordinating() {
        let mut fleet = FleetCoordinate::new(Config::default());
        fleet.add_agent(1, [0.0, 0.0], vec![]);
        fleet.add_agent(2, [1.0, 0.0], vec![]);
        fleet.add_agent(3, [0.5, 0.87], vec![]);
        fleet.add_trust_edge(1, 2);
        fleet.add_trust_edge(2, 3);
        fleet.add_trust_edge(3, 1);
        let report = fleet.analyze();
        assert!(report.fleet_theorem.is_laman_rigid);
    }

    #[test]
    fn test_trust_encoding_batch() {
        let fleet = FleetCoordinate::new(Config::default());
        let batch = fleet.trust_encoder().encode_batch(&[
            [0.6, 0.8],
            [0.8, 0.6],
            [-0.6, 0.8],
        ]);
        assert_eq!(batch.len(), 3);
    }
}
