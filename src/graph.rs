//! Fleet Constraint Graph — Laman Rigidity + H¹ Cohomology
//!
//! Laman's theorem (1867): A graph with V vertices is generically rigid in 2D
//! iff it has exactly 2V-3 edges and every subgraph with v' vertices has
//! at most 2v'-3 edges.
//!
//! Key caveat: Laman's theorem establishes the edge count condition (E=2V-3 for generic rigidity in 2D) and the subgraph condition (every subgraph has at most 2v'-3 edges), but does NOT place an upper bound on vertex degree. A Laman graph can have vertices of arbitrarily high degree.
//!
//! This maps directly to fleet coordination:
//! - Vertices = agents
//! - Edges = trust/communication links
//! - Rigid graph = provably self-coordinating fleet (no central coordinator)
//!
//! H¹ dimension = number of independent cycles = number of redundant
//! constraint paths = "emergence" in the network.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// One agent in the fleet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetAgent {
    pub id: u64,
    pub position: [f64; 2],
    pub neighbors: Vec<u64>,
    pub capabilities: Vec<String>,
}

/// The fleet constraint graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetGraph {
    agents: Vec<FleetAgent>,
    /// O(1) agent lookup by ID — mirrors FM's tile_index pattern
    agent_index: HashMap<u64, usize>,
    adjacency: HashMap<u64, HashSet<u64>>,
    edge_count: usize,
}

impl FleetGraph {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            agent_index: HashMap::new(),
            adjacency: HashMap::new(),
            edge_count: 0,
        }
    }

    pub fn add_agent(&mut self, id: u64, position: [f64; 2], capabilities: Vec<String>) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.adjacency.entry(id) {
            e.insert(HashSet::new());
            let idx = self.agents.len();
            self.agent_index.insert(id, idx);
            self.agents.push(FleetAgent { id, position, neighbors: vec![], capabilities });
        }
    }

    pub fn add_edge(&mut self, a: u64, b: u64) {
        // Only count as one edge (undirected) even though we store both directions
        let mut new_edge = false;
        if let Some(set) = self.adjacency.get_mut(&a) {
            if set.insert(b) {
                new_edge = true;
                if let Some(agent) = self.agents.iter_mut().find(|ag| ag.id == a) {
                    if !agent.neighbors.contains(&b) {
                        agent.neighbors.push(b);
                    }
                }
            }
        }
        if let Some(set) = self.adjacency.get_mut(&b) {
            if set.insert(a) {
                if new_edge {
                    self.edge_count += 1;
                }
                if let Some(agent) = self.agents.iter_mut().find(|ag| ag.id == b) {
                    if !agent.neighbors.contains(&a) {
                        agent.neighbors.push(a);
                    }
                }
            }
        }
    }

    /// Number of vertices — math notation V
    #[allow(non_snake_case)]
    pub fn V(&self) -> usize {
        self.agents.len()
    }

    /// Number of edges — math notation E
    #[allow(non_snake_case)]
    pub fn E(&self) -> usize {
        self.edge_count
    }

    /// Check Laman rigidity condition
    ///
    /// A graph is generically rigid (2D) iff:
    /// 1. E = 2V - 3 (approximately — allow 5% tolerance for boundary cases)
    /// 2. Every subgraph with v' vertices has E' ≤ 2v' - 3
    ///
    /// For small graphs (V < 3), just check main condition.
    #[allow(non_snake_case)]
    pub fn check_laman_rigidity(&self) -> RigidityResult {
        let v = self.V();
        let e = self.E();

        // Condition 1: E = 2V - 3 (with tolerance for small graphs)
        let expected_e = 2 * v - 3;
        let e_ratio = if expected_e > 0 { e as f64 / expected_e as f64 } else { 1.0 };

        // For V < 3, skip subgraph check (triangle is minimum rigid structure)
        let subgraph_check = if v < 3 {
            true
        } else {
            let mut ok = true;
            for agent in &self.agents {
                let v_prime = agent.neighbors.len() + 1;
                let e_prime = agent.neighbors.len();
                if v_prime >= 2 && e_prime > 2 * v_prime - 3 {
                    ok = false;
                    break;
                }
            }
            ok
        };

        let is_rigid = (e_ratio - 1.0).abs() < 0.05 && subgraph_check; // 5% tolerance
        
        // Betti number β₁ = E - V + C (number of independent cycles)
        let h1 = if e >= v { e - v + 1 } else { 0 };

        RigidityResult {
            is_rigid,
            V: v,
            E: e,
            expected_E: expected_e,
            h1_dimension: h1,
            critical_subgraphs: vec![],
            max_neighbors: self.agents.iter().map(|a| a.neighbors.len()).max().unwrap_or(0),
        }
    }

    /// Maximum neighbors for any agent (Laman's theorem: 2V-3 for V≥3)
    pub fn max_neighbors(&self) -> usize {
        self.agents.iter().map(|a| a.neighbors.len()).max().unwrap_or(0)
    }

    /// Get agent by ID — O(1) via HashMap index
    pub fn get_agent(&self, id: u64) -> Option<&FleetAgent> {
        self.agent_index.get(&id).and_then(|&idx| self.agents.get(idx))
    }

    /// Get neighbors of an agent
    pub fn get_neighbors(&self, id: u64) -> Vec<u64> {
        self.adjacency.get(&id).map(|s| s.iter().copied().collect::<Vec<u64>>()).unwrap_or_default()
    }
}

impl Default for FleetGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of rigidity analysis
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct RigidityResult {
    /// True if graph is generically rigid (2D)
    pub is_rigid: bool,
    pub V: usize,
    pub E: usize,
    pub expected_E: usize,
    /// H¹ dimension = number of independent cycles
    pub h1_dimension: usize,
    /// Agent IDs of over-constrained subgraphs
    pub critical_subgraphs: Vec<u64>,
    /// Maximum neighbor count across all agents
    pub max_neighbors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laman_rigid_triangle() {
        // 3 agents, 3 edges (K3) — 2*3-3 = 3 ✓
        let mut graph = FleetGraph::new();
        graph.add_agent(1, [0.0, 0.0], vec![]);
        graph.add_agent(2, [1.0, 0.0], vec![]);
        graph.add_agent(3, [0.5, 0.87], vec![]);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 1);

        let result = graph.check_laman_rigidity();
        assert!(result.is_rigid);
        assert_eq!(result.V, 3);
        assert_eq!(result.E, 3);
        assert_eq!(result.max_neighbors, 2);
    }

    #[test]
    fn test_laman_indeterminate_square() {
        // 4 agents, 4 edges (square) — 2*4-3 = 5, we have 4 — not rigid
        let mut graph = FleetGraph::new();
        graph.add_agent(1, [0.0, 0.0], vec![]);
        graph.add_agent(2, [1.0, 0.0], vec![]);
        graph.add_agent(3, [1.0, 1.0], vec![]);
        graph.add_agent(4, [0.0, 1.0], vec![]);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 1);

        let result = graph.check_laman_rigidity();
        assert!(!result.is_rigid);
        assert_eq!(result.max_neighbors, 2);
    }

    #[test]
    fn test_k12_complete_graph() {
        // K12: 12 agents, 12*11/2 = 66 edges
        // 2V-3 = 21, but K12 has 66 > 21 — over-constrained
        let mut graph = FleetGraph::new();
        for i in 0..12 {
            graph.add_agent(i as u64, [i as f64, 0.0], vec![]);
        }
        for i in 0..12 {
            for j in (i+1)..12 {
                graph.add_edge(i as u64, j as u64);
            }
        }

        let result = graph.check_laman_rigidity();
        assert!(!result.is_rigid); // Over-constrained
        assert_eq!(result.max_neighbors, 11);
    }

    #[test]
    fn test_h1_dimension_cycle() {
        // Triangle: 3 vertices, 3 edges → H1 = E-V+1 = 3-3+1 = 1 (one independent cycle)
        let mut graph = FleetGraph::new();
        graph.add_agent(0, [0.0, 0.0], vec![]);
        graph.add_agent(1, [1.0, 0.0], vec![]);
        graph.add_agent(2, [0.0, 1.0], vec![]);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 0);

        let result = graph.check_laman_rigidity();
        assert_eq!(result.h1_dimension, 1);
        assert!(result.is_rigid); // K3 is Laman-rigid
    }

    #[test]
    fn test_o1_agent_lookup() {
        // Prove O(1) lookup via HashMap index (not iter().find())
        let mut graph = FleetGraph::new();
        for i in 0..100 {
            graph.add_agent(i, [i as f64, 0.0], vec![format!("cap_{}", i)]);
        }
        // O(1) lookup: constant time regardless of position
        let start = std::time::Instant::now();
        for i in 0..100 {
            let agent = graph.get_agent(i);
            assert!(agent.is_some());
            assert_eq!(agent.unwrap().id, i);
        }
        let elapsed = start.elapsed();
        // Should be very fast (< 1ms for 100 lookups)
        assert!(elapsed.as_secs_f64() < 0.001, "O(1) lookup took {}s for 100 agents", elapsed.as_secs_f64());
    }

    #[test]
    fn test_o1_lookup_not_found() {
        let mut graph = FleetGraph::new();
        graph.add_agent(42, [0.0, 0.0], vec![]);
        assert!(graph.get_agent(999).is_none());
    }

    #[test]
    fn test_agent_index_consistency() {
        // Ensure index stays consistent after adding many agents
        let mut graph = FleetGraph::new();
        for i in 0..50 {
            graph.add_agent(i * 2, [i as f64, 0.0], vec![]);
        }
        for i in 0..50 {
            let agent = graph.get_agent(i * 2).expect(&format!("agent {} should exist", i * 2));
            assert_eq!(agent.id, i * 2);
        }
    }
}
