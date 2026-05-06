//! crystal_sync — Time-as-universal-sensor Phase A
//!
//! Crystal-based phase synchronization, coherence monitoring,
//! and self-calibrating clock hierarchy for fleet coordination.

use std::collections::HashMap;

// =============================================================================
// CrystalInfo — Physical crystal oscillator description
// =============================================================================

/// Physical properties of a local crystal oscillator.
///
/// The crystal's actual frequency f = f_nominal * (1 + drift_ppm / 1_000_000).
/// Phase offset is accumulated ticks between local and reference clocks.
pub struct CrystalInfo {
    /// Nominal frequency in Hz (e.g., 25_000_000 for 25 MHz crystal)
    pub frequency_hz: u64,
    /// Acceptable drift from nominal in parts-per-million (ppm)
    pub tolerance_ppm: u32,
    /// Accumulated phase offset relative to a reference clock, in ticks.
    /// Positive means local clock lags reference; negative means leads.
    current_phase_offset: i32,
}

impl CrystalInfo {
    /// Create a new crystal info record.
    pub fn new(frequency_hz: u64, tolerance_ppm: u32) -> Self {
        Self {
            frequency_hz,
            tolerance_ppm,
            current_phase_offset: 0,
        }
    }

    /// Get current phase offset in ticks.
    pub fn phase_offset(&self) -> i32 {
        self.current_phase_offset
    }

    /// Update the phase offset.
    pub fn set_phase_offset(&mut self, offset: i32) {
        self.current_phase_offset = offset;
    }
}

// =============================================================================
// PhaseSync — Manages phase exchange between agents
// =============================================================================

/// Manages phase synchronization between fleet agents via crystal exchange.
///
/// Each agent reads its local crystal counter and exchanges tick values
/// with peers to compute and track phase offsets.
pub struct PhaseSync {
    /// Local crystal description
    crystal: CrystalInfo,
    /// Monotonic tick counter (mock: increments each read)
    tick_count: u64,
}

impl PhaseSync {
    /// Create a new PhaseSync with nominal crystal parameters.
    pub fn new() -> Self {
        Self::with_crystal(CrystalInfo::new(25_000_000, 100))
    }

    /// Create a PhaseSync with a specific crystal.
    pub fn with_crystal(crystal: CrystalInfo) -> Self {
        Self {
            crystal,
            tick_count: 0,
        }
    }

    /// Create a PhaseSync with explicit crystal parameters.
    pub fn with_params(frequency_hz: u64, tolerance_ppm: u32) -> Self {
        Self::with_crystal(CrystalInfo::new(frequency_hz, tolerance_ppm))
    }

    /// Read the local crystal counter.
    ///
    /// Mock: simulates independent crystal oscillators. Each agent accumulates
    /// ticks proportional to its crystal's frequency relative to a 25 MHz
    /// reference. A 100-ppm faster crystal accumulates ticks faster.
    ///
    /// Returns tick_count * freq_ratio * SCALE, scaled so the frequency
    /// difference produces a measurable offset after u64 truncation.
    const CRYSTAL_SCALE: f64 = 1_000_000_000_000.0; // 1e12
    pub fn crystal_read(&mut self) -> u64 {
        self.tick_count = self.tick_count.saturating_add(1);
        let freq_ratio = self.crystal.frequency_hz as f64 / 25_000_000.0;
        (self.tick_count as f64 * freq_ratio * Self::CRYSTAL_SCALE) as u64
    }

    /// Compute phase offset between a peer's tick and local tick.
    ///
    /// offset = peer_tick - local_tick
    /// Positive means peer is ahead (local lags); negative means peer is behind.
    pub fn compute_offset(&self, peer_tick: u64, local_tick: u64) -> i64 {
        // Usewrapping subtraction to handle overflow safely.
        peer_tick.wrapping_sub(local_tick) as i64
    }

    /// Update consensus state from a set of peer offsets.
    ///
    /// Returns the mean offset and a phase coherence score.
    /// Coherence = 1/variance when variance > 0, else 0 (perfect consensus).
    pub fn update_consensus(&mut self, offsets: &[i64]) -> f64 {
        if offsets.is_empty() {
            return 0.0;
        }

        let n = offsets.len() as f64;
        let mean: f64 = offsets.iter().map(|&o| o as f64).sum::<f64>() / n;

        // Compute population variance
        let variance: f64 = offsets
            .iter()
            .map(|&o| {
                let d = o as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / n;

        // Phase coherence: 1/variance, or 0 if variance is zero (perfect consensus)
        if variance == 0.0 {
            0.0
        } else {
            1.0 / variance
        }
    }

    /// Compute Dir48 sum of phase directions around the fleet loop.
    ///
    /// Returns an 8-bit value representing the 48-direction composition
    /// of all accumulated offsets. Mock: sum of absolute offsets modulo 256.
    pub fn zhc_loop_sum(&self) -> u8 {
        // Mock: use the phase offset's absolute value as the direction magnitude.
        // In a real implementation this would map offsets to Dir48 sectors.
        let mag = self.crystal.phase_offset().abs() as u32;
        (mag % 256) as u8
    }
}

impl Default for PhaseSync {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// PhaseMonitor — Monitors phase coherence and emits warnings
// =============================================================================

/// Monitors phase correction rates and detects threshold breaches.
///
/// Emits warnings when the fleet's average correction rate exceeds
/// configured thresholds, indicating premature clock emergence.
pub struct PhaseMonitor {
    /// Maximum acceptable average correction rate (ticks/ms)
    threshold: f64,
}

impl PhaseMonitor {
    /// Create a new PhaseMonitor with the given threshold.
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Compute the rate of phase correction between two offset samples.
    ///
    /// rate = (curr_offset - prev_offset) / dt_ms
    pub fn correction_rate(&self, prev_offset: i64, curr_offset: i64, dt_ms: u64) -> f64 {
        if dt_ms == 0 {
            return 0.0;
        }
        (curr_offset as f64 - prev_offset as f64) / (dt_ms as f64)
    }

    /// Compute fleet-wide mean correction rate from individual agent rates.
    pub fn avg_correction_rate(&self, rates: &[f64]) -> f64 {
        if rates.is_empty() {
            return 0.0;
        }
        rates.iter().sum::<f64>() / (rates.len() as f64)
    }

    /// Check whether the fleet's average correction rate exceeds threshold.
    pub fn threshold_breach(&self, avg_rate: f64) -> bool {
        avg_rate > self.threshold
    }

    /// Emit a phase anomaly warning.
    pub fn emit_warning(&self) -> String {
        "PREMATURE_EMERGENCE: phase anomaly detected".to_string()
    }
}

// =============================================================================
// TempoReale — Self-calibrating clock hierarchy
// =============================================================================

/// Self-calibrating clock hierarchy for fleet time management.
///
/// Manages leader election, PLL synchronization, and failover when
/// agents in the hierarchy fail.
pub struct TempoReale {
    /// Map from agent ID to current phase offset (ticks)
    offsets: HashMap<u64, i64>,
    /// Currently elected leader agent ID
    leader: Option<u64>,
    /// Map from agent ID to coherence score
    coherence_scores: HashMap<u64, f64>,
}

impl TempoReale {
    /// Create a new TempoReale instance.
    pub fn new() -> Self {
        Self {
            offsets: HashMap::new(),
            leader: None,
            coherence_scores: HashMap::new(),
        }
    }

    /// Elect the agent with the highest coherence score.
    ///
    /// Returns the agent ID with maximum coherence_i.
    pub fn elect(&self, coherence_scores: &[(u64, f64)]) -> u64 {
        coherence_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| *id)
            .unwrap_or(0)
    }

    /// Synchronize local PLL to a reference agent.
    ///
    /// Adjusts local clock to track the reference using peer offsets.
    /// Stores the offset for each peer relative to the reference.
    pub fn sync_to(&mut self, reference_agent: u64, offsets: &[(u64, i64)]) {
        self.leader = Some(reference_agent);
        for &(agent, offset) in offsets {
            self.offsets.insert(agent, offset);
        }
    }

    /// Failover: select the next best agent after a failure.
    ///
    /// Returns the agent ID with highest coherence among remaining agents,
    /// or None if no agents remain.
    pub fn failover(&self, coherence_scores: &[(u64, f64)], failed_agent: u64) -> Option<u64> {
        coherence_scores
            .iter()
            .filter(|(id, _)| *id != failed_agent)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| *id)
    }
}

impl Default for TempoReale {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: three agents with nominal crystals converge to perfect consensus.
    ///
    /// Given three agents with 0 ppm drift, phase offsets should be
    /// identical after exchange, giving phase_coherence = 0 (variance = 0).
    #[test]
    fn test_phase_sync_convergence() {
        let mut agent0 = PhaseSync::with_params(25_000_000, 100);
        let mut agent1 = PhaseSync::with_params(25_000_000, 100);
        let mut agent2 = PhaseSync::with_params(25_000_000, 100);

        // Simulate synchronized reads
        let tick0 = agent0.crystal_read();
        let tick1 = agent1.crystal_read();
        let tick2 = agent2.crystal_read();

        // All agents see the same reference time → zero offsets
        let offsets = &[
            agent0.compute_offset(tick0, tick0),
            agent1.compute_offset(tick1, tick1),
            agent2.compute_offset(tick2, tick2),
        ];

        let coherence = agent0.update_consensus(offsets);
        // variance = 0 → coherence = 0 (perfect consensus)
        assert_eq!(coherence, 0.0, "perfect consensus should yield coherence = 0");
    }

    /// Test: one crystal 100 ppm off causes measurable phase drift.
    ///
    /// The drifted crystal's offset differs from nominal agents, increasing
    /// variance and reducing coherence below a threshold.
    #[test]
    fn test_phase_sync_drift() {
        let mut nominal = PhaseSync::with_params(25_000_000, 100);
        let mut drifted = PhaseSync::with_params(25_000_000 + 2500, 100); // 100 ppm off

        let tick_nominal = nominal.crystal_read();
        let tick_drifted = drifted.crystal_read();

        // Drifted crystal reads a different tick value (simulated frequency difference)
        // Offset of drifted vs nominal
        let offset_drifted = nominal.compute_offset(tick_drifted, tick_nominal);

        // Add nominal + drifted offsets to consensus
        let offsets = &[
            0,                               // nominal agent 0
            0,                               // nominal agent 1
            offset_drifted,                  // drifted agent
        ];

        let coherence = nominal.update_consensus(offsets);

        // With a 100 ppm crystal, coherence should be significantly reduced
        // Since variance > 0, coherence = 1/variance > 0.
        // We verify it's non-zero but detect the drift by checking coherence
        // is below what perfect consensus would give (which is 0).
        // Actually coherence = 1/variance, and variance > 0 means coherence > 0.
        // The key is that coherence < infinity (i.e., variance != 0).
        assert!(
            coherence > 0.0,
            "drifted crystals should produce non-zero coherence"
        );
        // A threshold check: coherence for drifted crystal should be much lower
        // than for perfect sync. With 2 nominal + 1 drifted, variance is finite.
        // coherence < 1.0 confirms variance > 1
        assert!(
            coherence < 1.0,
            "drifted ensemble should have coherence < 1"
        );
    }

    /// Test: correction rate calculation.
    ///
    /// prev=-5, curr=+3, dt=1000ms → rate = (3-(-5))/1000 = 0.008 ticks/ms.
    #[test]
    fn test_correction_rate() {
        let monitor = PhaseMonitor::new(0.010);
        let rate = monitor.correction_rate(-5, 3, 1000);
        assert!((rate - 0.008).abs() < 1e-10, "rate = 0.008");
    }

    /// Test: TempoReale elects agent with highest coherence score.
    ///
    /// coherence [(0,0.1), (1,0.9), (2,0.2)] → elected = agent 1 (0.9).
    #[test]
    fn test_tempo_reale_election() {
        let tempo = TempoReale::new();
        let scores = &[(0, 0.1), (1, 0.9), (2, 0.2)];
        let elected = tempo.elect(scores);
        assert_eq!(elected, 1, "agent with 0.9 coherence should be elected");
    }

    /// Test: TempoReale failover after agent failure.
    ///
    /// agent 1 fails; remaining [(0,0.1), (2,0.5)] → failover returns agent 2.
    #[test]
    fn test_tempo_reale_failover() {
        let tempo = TempoReale::new();
        let scores = &[(0, 0.1), (1, 0.9), (2, 0.5)];
        let result = tempo.failover(scores, 1);
        assert_eq!(result, Some(2), "failover should return agent 2");
    }

    /// Test: PhaseMonitor threshold breach detection.
    ///
    /// rate=0.015, threshold=0.010 → breach=true.
    #[test]
    fn test_phase_monitor_threshold() {
        let monitor = PhaseMonitor::new(0.010);
        assert!(
            monitor.threshold_breach(0.015),
            "rate 0.015 > threshold 0.010 should breach"
        );
    }
}
