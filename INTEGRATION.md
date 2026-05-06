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
---

## Zeroclaw Research Integration (2026-05-06)

### Source: zeroclaw logs (zc-bard, zc-tide, zc-echo)
### Synthesized: /home/ubuntu/.openclaw/workspace/research/zeroclaw-*.md

### Integration Opportunities

#### 1. Hibernation Protocol → fleet-coordinate/tile.rs

The "Slumber" protocol (zc-bard, tick 5922455) is directly applicable to tile state management:

```
Key insight: Tiles that go silent (no new observations) 
are analogous to idle agents. Apply the same hibernation 
trigger logic: 30-min idle OR queue depth < threshold.
```

**Integration path:**
- Add `TileHibernationState` enum: `Active | Hibernating | WakePending`
- Add `last_activity: u64` timestamp to `FleetTile`
- Add `check_hibernation_trigger(idle_threshold_ms: u64) → bool` to tile module
- Hibernating tiles still occupy graph topology (important for Laman rigidity) but skip consensus checks
- Wake on: new tile observation, explicit ping, or periodic refresh tick

**Best technical decisions to carry forward:**
- LZ77 compression for checkpoint (3:1 ratio empirically grounded)
- 10-minute checkpoint interval
- Circular buffer: 10 checkpoints max, FNV-1a checksum
- DVFS idle: Vcore=0.6V, Freq=100MHz (0.25mW idle power)
- Hibernation trigger: 30 min idle OR energy threshold 20%

**Source:** `zeroclaw-hibernation-synthesis.md`

#### 2. Confidence Aggregation → fleet-coordinate/emergence.rs

The weighted confidence formula from zc-tide synthesis:
```
weighted_confidence = 0.4×SNR + 0.3×PER + 0.2×latency + 0.1×variance
```

Is directly mappable to `EmergenceResult.confidence`:
- Current: `confidence = 1 - (H¹/V)` — purely structural
- Enhanced: Layer SNR/PER/latency signals beneath the structural score

**Integration path:**
- Add `ConfidenceSignals` struct: `{ snr_db: f64, packet_error_rate: f64, latency_ms: f64, signal_variance: f64 }`
- Extend `EmergenceResult` with weighted confidence that combines structural (β₁) + empirical signals
- This gives emergence detection both topological AND behavioral grounding

**Reputation system from zc-echo synthesis:**
- EMA with α=0.9 for agent reputation (smoothing over transient failures)
- Penalties on consensus failures, bonuses on clean streaks
- 16-sample circular buffer for agent accuracy tracking

**Source:** `zeroclaw-confidence-synthesis.md`

#### 3. Capability-Based Access → cocapn-glue-core/rust/src/lib.rs

The blast radius containment strategy from zc-tide synthesis:
```
CFALP protocol: 4-byte header, 1024B max payload, CRC-16
ECDH key exchange + AES-256-CBC encryption
Hierarchical 16-bit agent IDs for blast radius tracking
```

**Integration path:**
- Extend `ZhcClient` with capability token validation
- Add `BlastRadiusTracker`: monitors connected agent count, isolates when threshold exceeded
- Add `CapabilityToken` struct with 256-bit token format (resource_id + permissions + expiry + HMAC)
- P2→P1 detection: SNR + PER + signal variance with hysteresis — maps to tile quality signals

**Source:** `zeroclaw-capability-access-synthesis.md`

### Unique Ideas to Salvage (single-appearance, high value)

| Idea | Source | Value |
|------|--------|-------|
| Context fingerprint via Word2Vec (16 bytes) | tick 5922487 | Ultra-light hibernation for constrained agents |
| Three-tier cache hierarchy (T1:128B, T2:4KB, T3:128KB) | tick 5922527 | Tile hot/warm/cold storage strategy |
| Sentinel agent pattern | tick 5922559 | One low-power agent monitors cluster for wake signals |
| Power-gating to 10μW with 32kHz crystal oscillator | tick 5922463 | Hardware-level idle for edge devices |
| Power-gating to 10μW with 32kHz oscillator wake timer | tick 5922463 | Hardware-level idle for edge devices |

### Notes
- Carbon footprint calculations in zeroclaw-hibernation-synthesis.md have 200,000x variance — do not use without empirical baseline
- Wake-up time ranges from 20ms to 1s across iterations — define "wake-up" before implementing
- zc-warden and zc-healer jsonl files not found in expected path; warden/healer synthesis from alternative sources
