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

/// Maximum neighbors for rigidity (Laman's theorem: 2V-3)
pub const MAX_RIGID_NEIGHBORS: usize = 12;

/// Information content per trust vector (log₂ 48 ≈ 5.585 bits)
pub const TRUST_BITS_PER_VECTOR: f64 = 5.58496;
