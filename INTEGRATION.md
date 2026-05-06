# Fleet Coordinate + ABOracle: Cross-Pollination Integration

## Current State

### fleet-coordinate (newly created, 22 tests passing)
- **ZHC consensus**: 38ms latency, any Byzantine tolerance
- **Pythagorean48**: 48 exact direction vectors (5.585 bits/vector)
- **Laman rigidity**: E = 2V-3 for provably self-coordinating fleets
- **H¹ emergence**: β₁ = E - V + C, detects over-constraint
- **Beam joint equilibrium**: Newton-Raphson in R^{4(N-1)}

### aboracle (existing, FM-instinct architecture)
- **Instinct stack**: SURVIVE > FLEE > GUARD > HOARD > COOPERATE > EVOLVE
- **Pythagorean48**: 22 triples only (incomplete codebook)
- **6-layer ship protocol**: Harbor/TidePool/Current/Channel/Beacon/Reef
- **Trust weighted**: Casey(1.0) > FM(0.85) > subagents
- **Beachcomb**: research + holonomy checking
- **Fleet-heartbeat**: mycorrhizal routing for FM coordination

## Integration Opportunities

### 1. Unified Pythagorean48 Codebook
**Problem**: aboracle uses 22 triples, fleet-coordinate uses 48. Different codebooks = can't verify each other's encodings.

**Solution**: Merge to full 48-direction codebook in fleet-coordinate, export as a shared library.

**Action**: Update aboracle's `PYTHAGOREAN_TRIPLES` to use `fleet_coordinate::pythagorean48::TrustVector::all_directions()` via a PyO3 binding, OR publish the 48 triples as a shared JSON file in `SuperInstance/fleet-math`.

### 2. ABOracle Instinct Model → Fleet Trust Topology
FM's instinct levels map to trust weights:
- SURVIVE (critical) → trust_weight = 1.0
- FLEE (high) → trust_weight = 0.85
- GUARD (normal) → trust_weight = 0.70
- HOARD (low) → trust_weight = 0.55
- COOPERATE → trust_weight = 0.40
- EVOLVE (idle) → trust_weight = 0.25

This creates a **gradient** of trust that can be encoded in Pythagorean48 directions.

**Action**: Write `instinct_to_trust.rs` — converts FM instinct state to trust vectors, maps to fleet-coordinate's `TrustTopology`.

### 3. ZHC Consensus for ABOracle's Fleet-Heartbeat
ABOracle's FM monitor uses simple polling + trust-weighted response. It could use fleet-coordinate's formal ZHC consensus for Byzantine-tolerant coordination.

**Action**: Add `ZhcConsensus::check_andRespond(discussion_id)` to `fm_monitor.py` — instead of polling, subscribe to consensus state changes.

### 4. MUD-world Laman Rigidity Check
ABOracle's mud-agent bridges MUD world ↔ PLATO. The MUD world has agents (players/npcs) connected by communication links. We could apply Laman rigidity analysis to detect when the MUD world becomes "rigid" (self-coordinating without central control).

**Action**: Add `mud_rigidity_check.py` — periodically check MUD world graph (V agents, E connections), report rigidity status to PLATO.

### 5. Emergence Detection for Fleet Health
ABOracle's health-system monitors services. It could use H¹ emergence detection to predict service failures before they happen — when the service dependency graph becomes over-constrained (emergent behavior = cascade failure).

**Action**: Add `emergence_health_check.py` — map service dependencies as a graph, detect emergence before cascade.

## Proposed Integration Repo: `superinstance/fleet-agents`

Create a new meta-repo that integrates:
1. `fleet-coordinate` (Rust) — mathematical core
2. `aboracle` (Python) — FM-instinct agents
3. `plato-sdk` — PLATO integration
4. `cocapn-glue-core` — Keeper protocol

**Entry point**: `agents.py` — Python bindings to fleet-coordinate via PyO3.

## Quick Wins (Today)

1. **Sync Pythagorean48 triples** — write 48 triples to `fleet-math/pythagorean48-codes.json`, update aboracle's researcher.py to use it
2. **ABOracle health-system uses H¹ emergence** — add to monitor.py
3. **Mud-agent uses Laman rigidity** — add graph check to mud_bridge.py

## Files to Create

```
fleet-agents/
├── README.md
├── pythagorean48-codes.json  # 48 exact directions
├── instinct_trust_map.rs     # FM instinct → trust vector
├── emergence_health.rs      # H¹ emergence for service health
├── mud_rigidity.rs           # Laman rigidity for MUD world
└── src/
    └── lib.rs (re-exports fleet-coordinate)
```

## Mathematical Link

The core insight connecting fleet-coordinate and aboracle:

**Instincts are trust gradients. Trust gradients are Pythagorean48 vectors. Pythagorean48 vectors form a Laman-rigid topology. A Laman-rigid topology is provably self-coordinating (ZHC consensus).**

```
FM Instinct → Trust Weight → Pythagorean48 Vector → Laman Graph → ZHC Consensus
```

This means: ABOracle's entire instinct architecture can be formalized as a geometric constraint satisfaction problem — no ML needed.