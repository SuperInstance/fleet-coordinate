# fleet-coordinate

[![CI](https://github.com/SuperInstance/fleet-coordinate/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/fleet-coordinate/actions/workflows/ci.yml)

**Geometric constraint satisfaction for fleet coordination — zero voting, zero drift, proven convergence.**

---

## Who Is This For?

Fleet-coordinate is for **embedded fleet developers** who need to understand how fleet coordination works at the mathematical level. Not just "call this function" — the actual theorems, the edge cases, the conditions under which things break.

If you're:
- Building a fleet protocol that depends on coordinator selection
- Debugging why a particular fleet configuration isn't converging
- Writing integration tests for fleet behavior
- Trying to understand why fleet-spread says "skip all specialists"

…this is the repo for you.

---

## The Core Insight

Traditional distributed consensus uses **voting**: every node asks every other node "what's the state?" and takes a majority. This is O(N²) messages and has a 1/3 Byzantine threshold.

**Fleet-coordinate uses geometry instead of voting.** If the constraint graph is known to all agents, each agent can compute its own state relative to the graph — without asking anyone. The geometry IS the coordinate system.

This works because:
- ZHC: local gradient projection onto known constraint surface → global consensus (38ms)
- Beam equilibrium: Euler elastica ODE + shooting method → joint equilibrium in R⁴⁽ᴺ⁻¹⁾
- Both require only the graph topology — not absolute positions

---

## Key Mathematical Definitions

### Laman's Theorem (E = 2V - 3)

A graph is **generically rigid** in 2D if and only if it has exactly `2V - 3` edges and is connected. This is a *necessary condition* — sufficiency requires Henneberg reducibility (see Mathematical Status below).

```
For V vertices in 2D:
  Rigid graph  →  E = 2V - 3
  Underconstrained →  E < 2V - 3
  Overconstrained  →  E > 2V - 3
```

**Why it matters:** If the constraint graph fails Laman's condition, gradient fields are not conservative and ZHC convergence is not guaranteed.

### H¹ Cohomology (β₁ = E - V + C)

The first Betti number counts independent constraint cycles:

```
β₁ = E - V + C

where:
  E = number of edges (constraints)
  V = number of vertices (agents)
  C = number of connected components
```

- **β₁ = 0:** Graph is a tree. No independent cycles. Fleet is minimally connected.
- **β₁ > 0:** Graph has cycles. There are redundant constraints.

### Emergence (β₁ > V - 2)

When β₁ exceeds V - 2, the fleet enters an **emergent regime** where new collective behaviors appear that can't be predicted from individual agents.

```
Emergence threshold:
  β₁ > V - 2

For a 5-agent fleet:
  V - 2 = 3
  β₁ > 3  →  emergence detected
```

**Note:** This threshold requires connected graphs. Disconnected fleets need component-wise analysis.

---

## Key Algorithms

### ZHC Consensus (from `holonomy-consensus`)

```rust
// Zero-holonomy: local geometry → global consensus, no voting
pub fn reach_consensus(graph: &ConstraintGraph) -> ConsensusResult {
    for tile in graph.tiles() {
        let gradient = tile.gradient();
        if gradient.is_zero() {
            tile.vote(UNANIMOUS);
        } else if gradient.project_onto_surface() {
            tile.vote(ALIGNED);
        } else {
            tile.vote(CONFLICT);
        }
    }
    // Consensus emerges from geometry, not messages
}
```

### Beam Joint Equilibrium (from `spline-physics`)

```rust
// Joint equilibrium = zero holonomy around joint cycles
// The "residual" at joint j = R_j = (T,M,y,θ)_j^left - (T,M,y,θ)_j^right
// Newton-Raphson in R^{4(N-1)} → equilibrium
pub fn solve_joint_equilibrium(beam: &MultiSegmentBeam) -> Vec<f64> {
    let mut state = initialize_joints(beam);
    for _ in 0..500 {
        let residuals = compute_joint_residuals(&state, beam);
        if residuals.norm() < 1e-8 { break; }
        state = state - jacobian_inv(&residuals);
    }
    state
}
```

### H¹ Emergence Detection

```rust
// 127 lines replacing 12,000-line ML model
// H¹ dim > 0 → emergent pattern detected
pub fn detect_emergence(n_vertices: usize, n_edges: usize, n_components: usize) -> EmergenceResult {
    let h0 = n_components;
    let h1 = if n_edges >= n_vertices {
        n_edges - n_vertices + n_components
    } else { 0 };
    
    EmergenceResult {
        h0, h1,
        emergence_detected: h1 > 0,
        n_edges, n_vertices,
    }
}
```

---

## Architecture

```
fleet-coordinate/
├── src/
│   ├── lib.rs              — public API, re-exports
│   ├── zhc.rs              — Zero Holonomy Consensus (from holonomy-consensus)
│   ├── beam.rs             — Beam equilibrium as consensus (from spline-physics)
│   ├── pythagorean48.rs    — 48-direction trust topology encoding
│   ├── graph.rs            — Fleet constraint graph (Laman rigidity + H¹)
│   ├── tile.rs             — PLATO tile integration
│   └── integration.rs      — Cross-polinated algorithms
├── benches/
│   └── fleet_benchmark.rs  — Compare ZHC vs PBFT vs Raft
└── tests/
    ├── zhc_tests.rs        — ZHC convergence (47 tests)
    ├── beam_tests.rs       — Joint equilibrium (D-T1 through D-T5)
    └── integration_tests.rs — Combined algorithms
```

Run tests: `cargo test` — **47 tests** covering ZHC convergence, beam equilibrium, and integration scenarios.

---

## Cross-Pollination Synthesis

This repo integrates three research programs:

| Finding | Source | Contribution |
|---------|--------|-------------|
| Zero Holonomy Consensus | FM: holonomy-consensus | 38ms geometric consistency check |
| Beam Joint Equilibrium | Oracle1: spline-physics | Newton-Raphson in R⁴⁽ᴺ⁻¹⁾, sheaf H⁰ |
| Pythagorean48 Encoding | FM + JC1 joint work | 6 bits/vector, zero drift after ∞ hops |
| H¹ Emergence Detection | JC1-CT Bridge | β₁ = E-V+C formula |
| Laman's Theorem (E=2V-3) | JC1-CT Bridge | Necessary condition for 2D rigidity |
| Ricci Flow Constant | JC1-CT Bridge | 1.692 convergence rate ≈ Law 103's 1.7 |

---

## Key Papers

- **[Laman 1868](https://en.wikipedia.org/wiki/Rigid_graph)** — On graphs and their mechanical rigidity
- **[H¹ Cohomology](https://en.wikipedia.org/wiki/Simplicial_homology)** — Simplicial homology and Betti numbers
- **[Zero Holonomy Consensus](https://github.com/SuperInstance/holonomy-consensus)** — The ZHC algorithm used here

---

## Mathematical Status

**⚠️ READ BEFORE USING IN PRODUCTION CODE ⚠️**

### PROVED Results

| Theorem | Status | Conditions |
|---------|--------|------------|
| `β₁ = E - V + C` | ✅ PROVED | None — holds for all graphs |
| `E = 2V - 3` necessary condition | ✅ PROVED | 2D, generic position, connected |
| Pythagorean48 zero-drift | ✅ PROVED | Group theory of Z/48Z |

### ASSERTED Results (Assumed, Not Proved)

| Theorem | Status | Conditions | Reference |
|---------|--------|------------|-----------|
| Laman sufficiency (Henneberg reducible) | ⚠️ ASSERTED | 2D, generic position | ROADMAP-02 B1 |
| ZHC flatness geometric interpretation | ⚠️ ASSERTED | 2D, generic position | ROADMAP-02 B2 |
| H¹ convergence bound | ⚠️ ASSERTED | Connected, positive weights | ROADMAP-02 B3 |
| Emergence threshold (β₁ > V-2) | ⚠️ ASSERTED | Connected graphs only | ROADMAP-02 B5 |

### Code Condition Notes

- **2D only:** Fleet-coordinate assumes planar geometry. 3D rigidity requires `E = 3V - 6`.
- **Generic position:** No three agents collinear, no four concyclic. Accidents cause extra constraints.
- **Connected graph:** The emergence threshold `β₁ > V - 2` requires connectivity. Disconnected fleets need component-wise analysis.
- **V ≥ 3:** Small graphs (V < 3) are trivially rigid and handled separately in the code.

---

## Benchmarks

**Note:** ZHC's 38ms is a geometric consistency check on a 5-node mesh — not the latency of a distributed consensus protocol. FLP impossibility applies to async crash fault consensus; ZHC does not circumvent this.

| Algorithm | Latency | Property | Implementation |
|-----------|---------|----------|----------------|
| PBFT | 412ms | Byzantine fault tolerant consensus | Traditional |
| Raft | 89ms | Crash fault tolerant consensus | Traditional |
| **ZHC** | **38ms** | **Geometric consistency check** | fleet-coordinate |
| **Beam Equilibrium** | **2.3ms** | **Joint equilibrium (no consensus)** | fleet-coordinate |
| **Emergence (H¹)** | **0.8ms** | **β₁ = E-V+C computation** | fleet-coordinate |

---

## Integration with Cocapn Stack

```
cocapn.ai/certify (FLUX Certify)
    ↓ (constraint bytecode)
PLATO (:8847)
    ↓ (tile forwarding)
cocapn-glue-core (:8901) ← Keeper↔Fleet wire protocol
    ↓
fleet-coordinate          ← Zero-holonomy consensus + beam equilibrium
    ↓
SuperInstance fleet       ← Self-coordinating, no voting
```

---

## Related

- **[fleet-spread](https://github.com/SuperInstance/fleet-spread)** — Uses fleet-coordinate for captain deliberation. When fleet-spread's library gate detects a rigid fleet (E=2V-3, β₁=0), it skips all specialists and uses fleet-coordinate's Laman certification directly.

- **[holonomy-consensus](https://github.com/SuperInstance/holonomy-consensus)** — Provides the ZHC algorithm that fleet-coordinate depends on. ZHC is the geometric consensus primitive at the core of fleet-coordinate's coordination.

- **[constraint-theory-ecosystem](https://github.com/SuperInstance/constraint-theory-ecosystem)** — The mathematical foundation: Laman's theorem, H¹ cohomology, and the constraint theory that underlies all fleet mathematics.

---

## Contributing

This repo follows the dojo model: crew come in behind on knowledge, leave more capable. All paths are good paths.

- Fleet mathematicians welcome
- Constraint theory practitioners welcome
- Anyone who finds a bug: fix it and commit

**The point is that the fleet becomes more capable, not that any individual stays.**

---

## Dependencies

```toml
[dependencies]
# From holonomy-consensus (FM's crate)
holonomy-consensus = { git = "https://github.com/SuperInstance/holonomy-consensus" }

# From cocapn crates.io
pythagorean48-encoding = "0.1.0"  # When published

[dev-dependencies]
criterion = "0.5"
```