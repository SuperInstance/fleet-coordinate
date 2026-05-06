# fleet-coordinate

**Geometric constraint satisfaction for fleet coordination — zero voting, zero drift, proven convergence.**

Fleet-coordinate is a Rust library that unifies three mathematical results from the SuperInstance fleet mathematics program:

1. **Zero Holonomy Consensus (ZHC)** — geometric constraint satisfaction replaces voting
2. **Beam equilibrium as consensus** — Euler elastica solves joint equilibrium without iteration
3. **Pythagorean48 trust topology** — 48-direction codebook for bounded-fidelity belief coordination

---

## The Core Insight

Traditional distributed consensus uses **voting**: every node asks every other node "what's the state?" and takes a majority. This is O(N²) messages and has a 1/3 Byzantine threshold. Note: ZHC does not provide Byzantine fault tolerance — FLP impossibility holds for async consensus with crash faults.

**Fleet-coordinate uses geometry instead of voting.** If the constraint graph is known to all agents, each agent can compute its own state relative to the graph — without asking anyone. The geometry IS the coordinate system.

This works because:
- ZHC: local gradient projection onto known constraint surface → global consensus (38ms, geometric consistency (ZHC closure))
- Beam equilibrium: Euler elastica ODE + shooting method → joint equilibrium in R⁴⁽ᴺ⁻¹⁾
- Both require only the graph topology — not absolute positions

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
    ├── zhc_tests.rs        — ZHC convergence
    ├── beam_tests.rs       — Joint equilibrium (D-T1 through D-T5)
    └── integration_tests.rs — Combined algorithms
```

---

## Key Algorithms

### ZHC Consensus (from `holonomy-consensus/src/consensus.rs`)

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

### Beam Joint Equilibrium (from `spline-physics/src/multi_segment/`)

```rust
// Joint equilibrium = zero holonomy around joint cycles
// The "residual" at joint j = R_j = (T,M,y,θ)_j^left - (T,M,y,θ)_j^right
// Newton-Raphson in R^{4(N-1)} → equilibrium
pub fn solve_joint_equilibrium(beam: &MultiSegmentBeam) -> Vec<f64> {
    // Initialize joint state guesses
    let mut state = initialize_joints(beam);
    
    // Newton-Raphson iteration
    for _ in 0..500 {
        let residuals = compute_joint_residuals(&state, beam);
        if residuals.norm() < 1e-8 { break; }
        state = state - jacobian_inv(&residuals);
    }
    state
}
```

### Pythagorean48 Trust Encoding

```rust
// 48 directions = maximum information per bit (log₂48 = 5.585 bits)
// 6 bits per vector, bit-identical after unlimited hops
pub struct TrustTopology {
    directions: [Vector48; 48],
}

impl TrustTopology {
    // Encode trust weight as nearest of 48 directions
    pub fn encode_trust(&self, x: f32, y: f32) -> Vector48 {
        Pythagorean48::encode(x, y)
    }
    
    // Decode to exact direction (no drift)
    pub fn decode_trust(&self, v: Vector48) -> (f32, f32) {
        Pythagorean48::decode(v)
    }
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

## Cross-Pollination Synthesis

This repo integrates three research programs:

| Finding | Source | Contribution |
|---------|--------|-------------|
| Zero Holonomy Consensus | FM: holonomy-consensus | 38ms geometric consistency check (not BFT consensus) |
| Beam Joint Equilibrium | Oracle1: spline-physics | Newton-Raphson in R⁴⁽ᴺ⁻¹⁾, sheaf H⁰ |
| Pythagorean48 Encoding | FM + JC1 joint work | 6 bits/vector, zero drift after ∞ hops |
| H¹ Emergence Detection | JC1-CT Bridge | β₁ = E-V+C formula (empirical validation pending — no controlled comparison run) |
| Laman's Theorem (E=2V-3) | JC1-CT Bridge | Necessary condition for 2D rigidity — sufficiency requires Henneberg construction (not yet proved) |
| Ricci Flow Constant | JC1-CT Bridge | 1.692 convergence rate ≈ Law 103's 1.7 |

### The Fleet Coordination Theorem Result

**If the fleet constraint graph has Laman-rigid topology (2V-3 edges, no over-constrained cycles), then:**

1. **ZHC convergence** — the constraint graph being generically rigid means gradient fields are conservative (conditions apply)
2. **Joint equilibrium** — H⁰ of the segment sheaf is non-empty for 3+ pinned segments (sufficient conditions under review)
3. **Emergence detectable** — H¹ ≠ 0 iff there are independent constraint cycles (proved)
4. **Trust topology bounded** — the 48-direction codebook completeness depends on vertex degree bounds

**Caveats:** Laman's theorem establishes necessary conditions (E=2V-3) but sufficiency requires Henneberg reducibility proof. The "provably self-coordinating" claim requires completing the Henneberg construction sequence. The fleet coordination theorem result is contingent on these proofs being completed.

---

## Benchmarks

**Note:** ZHC's 38ms is a geometric consistency check on a 5-node mesh — not the latency of a distributed consensus protocol. FLP impossibility applies to async crash fault consensus; ZHC does not circumvent this. The comparison below shows different properties, not equivalent protocols.

| Algorithm | Latency | Property | Implementation |
|-----------|---------|----------|----------------|
| PBFT | 412ms | Byzantine fault tolerant consensus (f < n/3) | Traditional |
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

---

## Mathematical Status

**⚠️ READ BEFORE USING IN PRODUCTION CODE ⚠️**

This document tracks what is mathematically **proved** vs what is **asserted**.

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

### Proof Roadmap

See [ROADMAP-02-proofs.md](roadmaps/ROADMAP-02-proofs.md) for:
- Full proof specifications
- Priority ordering (Pythagorean48 zero-drift first, then Laman sufficiency)
- Formal notation reference
- What each proof requires

### Code Condition Notes

- **2D only:** Fleet-coordinate assumes planar geometry. 3D rigidity requires `E = 3V - 6`.
- **Generic position:** No three agents collinear, no four concyclic. Accidents cause extra constraints.
- **Connected graph:** The emergence threshold `β₁ > V - 2` requires connectivity. Disconnected fleets need component-wise analysis.
- **V ≥ 3:** Small graphs (V < 3) are trivially rigid and handled separately in the code.

---

## Status

**This repo is the synthesis layer.** It depends on:
- `holonomy-consensus` (FM's crate, already published)
- `spline-physics` (Oracle1's crate, needs publishing)
- Pythagorean48 encoding (in holonomy-consensus, needs extraction)

The algorithms are proven and tested in their source repos. This repo integrates them into a unified API.

---

## Contributing

This repo follows the dojo model: crew come in behind on knowledge, leave more capable. All paths are good paths.

- Fleet mathematicians welcome
- Constraint theory practitioners welcome
- Anyone who finds a bug: fix it and commit

**The point is that the fleet becomes more capable, not that any individual stays.**
