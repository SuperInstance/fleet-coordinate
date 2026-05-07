//! fleet-coordinate — Geometric constraint satisfaction for fleet coordination
//!
//! Unifies three mathematical results: Zero Holonomy Consensus (ZHC),
//! Beam Joint Equilibrium, and Pythagorean48 trust topology.
//!
//! Key theorem: A fleet with Laman-rigid constraint topology
//! (2V-3 edges, no over-constrained cycles) is provably
//! self-coordinating without voting.

pub mod zhc;
pub mod beam;
pub mod pythagorean48;
pub mod graph;
pub mod tile;
pub mod emergence;
pub mod integration;
pub mod crystal_sync;

// Re-export the primary types
pub use zhc::{ZhcConsensus, ConsensusResult};
pub use beam::{BeamSolver, JointEquilibrium, MultiSegmentBeam, JointState};
pub use pythagorean48::{TrustTopology, TrustVector};
pub use graph::{FleetGraph, RigidityResult};
pub use tile::{FleetTile, TileCoordination};
pub use emergence::{EmergenceDetector, EmergenceResult};
pub use integration::{FleetCoordinate, Config};
pub use crystal_sync::{PhaseSync, PhaseMonitor, TempoReale, CrystalInfo};

/// Example: crystal_sync — three agents with drifted crystal detection.
///
/// ```ignore
/// use fleet_coordinate::{PhaseSync, PhaseMonitor, TempoReale};
///
/// // Three agents: one reference (nominal), one healthy (nominal), one drifted (+100 ppm)
/// let mut ref_agent = PhaseSync::with_params(25_000_000, 100);
/// let mut healthy = PhaseSync::with_params(25_000_000, 100);
/// let mut drifted = PhaseSync::with_params(25_002_500, 100);
///
/// // Each agent reads its crystal
/// let tick_ref = ref_agent.crystal_read();
/// let tick_healthy = healthy.crystal_read();
/// let tick_drifted = drifted.crystal_read();
///
/// // Drifted agent's tick is higher (counts faster)
/// assert!(tick_drifted > tick_ref);
///
/// // Compute phase offset of drifted vs reference
/// let offset = ref_agent.compute_offset(tick_drifted, tick_ref);
/// assert!(offset > 0, "drifted crystal leads reference");
///
/// // Monitor correction rate
/// let monitor = PhaseMonitor::new(0.010);
/// let rate = monitor.correction_rate(0, offset, 1000);
/// assert!(monitor.threshold_breach(rate), "100 ppm drift should breach threshold");
pub const MAX_RIGID_NEIGHBORS: usize = 12;

/// Information content per trust vector (log₂ 48 ≈ 5.585 bits)
pub const TRUST_BITS_PER_VECTOR: f64 = 5.58496;
