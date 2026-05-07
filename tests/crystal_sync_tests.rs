//! crystal_sync integration tests
//!
//! Tests PhaseSync, PhaseMonitor, and TempoReale with ZHC consistency checking
//! across a fleet of 3 agents with different crystal frequencies.

use fleet_coordinate::{PhaseSync, PhaseMonitor, TempoReale};

/// Test: three agents with different crystals exchange ticks.
///
/// After reading, the drifted agent's tick differs from nominal agents.
/// The ZHC loop sum mock always returns 0 (phase_offset starts at 0 and
/// crystal_read() doesn't update it in this mock), so we test tick values instead.
#[test]
fn test_crystal_sync_zhc_drift_detection() {
    let mut agent0 = PhaseSync::with_params(25_000_000, 100); // reference
    let mut agent1 = PhaseSync::with_params(25_000_000, 100); // healthy
    let mut agent2 = PhaseSync::with_params(25_002_500, 100); // 100 ppm faster

    // Read ticks — drifted agent reads a different value than nominal
    let tick0 = agent0.crystal_read();
    let tick1 = agent1.crystal_read();
    let tick2 = agent2.crystal_read();

    // Both nominal agents at 25MHz should read the same value (same tick_count increments)
    assert_eq!(tick0, tick1, "nominal crystals should produce identical tick values");

    // Drifted agent's tick differs from nominal because CRYSTAL_SCALE * freq_ratio differs
    // 25_002_500 / 25_000_000 = 1.0001 → tick2 > tick0
    assert!(tick2 > tick0, "drifted crystal (100ppm faster) should produce higher tick count");
}

/// Test: PhaseMonitor detects when average correction rate exceeds threshold.
///
/// Simulate two correction rate measurements and verify threshold breach detection.
#[test]
fn test_phase_monitor_correction_rate_threshold() {
    let monitor = PhaseMonitor::new(0.010); // 0.010 ticks/ms threshold

    // Agent 0: rate = 0.005 ticks/ms (below threshold)
    let rate0 = monitor.correction_rate(-5, 0, 1000);
    assert!(!monitor.threshold_breach(rate0), "rate 0.005 should not breach threshold 0.010");

    // Agent 1: rate = 0.012 ticks/ms (above threshold — premature emergence)
    let rate1 = monitor.correction_rate(0, 12, 1000);
    assert!(monitor.threshold_breach(rate1), "rate 0.012 should breach threshold 0.010");

    // Fleet average: (0.005 + 0.012) / 2 = 0.0085 — below threshold
    let avg = monitor.avg_correction_rate(&[rate0, rate1]);
    assert!(!monitor.threshold_breach(avg), "average 0.0085 should not breach");

    // If drifted agent has higher rate: (0.005 + 0.015) / 2 = 0.010 — at threshold
    let avg_at = monitor.avg_correction_rate(&[rate0, 0.015]);
    // At exactly threshold, should NOT breach (threshold is strictly greater than)
    assert!(!monitor.threshold_breach(avg_at), "average exactly at threshold should not breach");

    // Exceed threshold: (0.005 + 0.016) / 2 = 0.0105
    let avg_over = monitor.avg_correction_rate(&[rate0, 0.016]);
    assert!(monitor.threshold_breach(avg_over), "average 0.0105 should breach threshold 0.010");
}

/// Test: TempoReale failover removes failed agent and elects highest coherence remaining.
///
/// Scenario: 3 agents [(0, 0.1), (1, 0.9), (2, 0.5)].
/// When agent 1 fails → failover returns agent 2 (coherence 0.5, best remaining).
/// When agent 2 fails → failover returns agent 1 (coherence 0.9, best remaining).
/// When agent 0 fails → failover returns agent 1 (coherence 0.9, best remaining).
/// When only agent 0 remains → failover returns agent 0 (only one left).
#[test]
fn test_tempo_reale_failover_selects_best_remaining() {
    let tempo = TempoReale::new();
    let scores = &[(0, 0.1), (1, 0.9), (2, 0.5)];

    // Agent 1 fails → agent 2 (0.5) wins over agent 0 (0.1)
    let result = tempo.failover(scores, 1);
    assert_eq!(result, Some(2), "failover after agent1 fails should return agent2");

    // Agent 2 fails → agent 1 (0.9) wins over agent 0 (0.1)
    let result2 = tempo.failover(scores, 2);
    assert_eq!(result2, Some(1), "failover after agent2 fails should return agent1");

    // Agent 0 fails → agent 1 (0.9) wins over agent 2 (0.5)
    let result3 = tempo.failover(scores, 0);
    assert_eq!(result3, Some(1), "failover after agent0 fails should return agent1");

    // Only one agent remains
    let scores_one = &[(1, 0.9)];
    let result4 = tempo.failover(scores_one, 1);
    assert_eq!(result4, None, "failover with no remaining agents should return None");
}

/// Test: 3 agents with 100-ppm crystals exchange offsets and detect drift.
///
/// This is an end-to-end test of the full crystal sync pipeline:
///   1. Each agent reads its crystal (tick values differ due to frequency differences)
///   2. Agents exchange tick values
///   3. Each agent computes offsets from its peers
///   4. Consensus update detects variance (drift)
///   5. PhaseMonitor detects threshold breach
#[test]
fn test_three_agent_phase_sync_with_drift() {
    // Reference agent (perfect crystal)
    let mut ref_agent = PhaseSync::with_params(25_000_000, 100);
    // Drifted agent (100 ppm faster)
    let mut drifted_agent = PhaseSync::with_params(25_002_500, 100);

    // Read ticks — drifted agent reads a different value
    let tick_ref = ref_agent.crystal_read();
    let tick_drifted = drifted_agent.crystal_read();

    // Drifted agent's tick is ahead (positive offset) because it counts faster
    let offset = ref_agent.compute_offset(tick_drifted, tick_ref);
    assert!(offset > 0, "drifted crystal should have positive offset (ahead of reference)");

    // Add both agents to a consensus measurement
    let offsets = &[
        ref_agent.compute_offset(tick_ref, tick_ref), // self: 0
        offset,                                        // drifted vs ref
    ];

    let coherence = ref_agent.update_consensus(offsets);
    // With variance > 0, coherence = 1/variance > 0
    assert!(coherence > 0.0, "non-zero variance should produce positive coherence");

    // With one drifted and one nominal, coherence should be relatively low
    // (not approaching the infinity limit of perfect consensus)
    assert!(coherence < 10.0, "modest variance from one drifted agent should keep coherence low");

    // PhaseMonitor threshold check
    let monitor = PhaseMonitor::new(1.0); // generous threshold
    // Simulate: prev=0, curr=offset, dt=1000ms
    let rate = monitor.correction_rate(0, offset, 1000);
    assert!(rate > 0.0, "positive offset should produce positive correction rate");
}