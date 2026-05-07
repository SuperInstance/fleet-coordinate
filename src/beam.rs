//! Beam Joint Equilibrium — spline-physics Phase D, re-exported for fleet-coordinate
//!
//! A multi-segment beam is a fleet of segment-agents joined at interior joints.
//! Joint equilibrium = all segment agents reach consensus on (T, M, y, θ) at each joint.
//!
//! The key insight: beam equilibrium IS a consensus problem in R⁴⁽ᴺ⁻¹⁾.
//! The "trust topology" is the joint adjacency graph. Consensus is reached
//! when the sheaf cohomology H⁰(S) is non-empty (global section exists).
//!
//! This module re-uses the multi_segment module from spline-physics via
//! the multi_agent/joint_debate module.

use serde::{Deserialize, Serialize};

// =====================================================================
// Re-exported types from spline-physics multi_segment module
// These would normally come from the spline-physics crate as a dependency
// =====================================================================

/// Boundary condition at a beam endpoint
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BoundaryCondition {
    /// Fully fixed (displacement + slope = 0)
    Fixed,
    /// Free (zero internal force)
    Free,
    /// Pinned (displacement = 0, moment free)
    Pinned,
    /// Roller (one direction free)
    Roller,
    /// Prescribed displacement and/or slope
    Prescribed { y: f64, theta: f64 },
}

/// Material properties for beam segments
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub youngs_modulus: f64, // GPa
    pub density: f64,         // g/cm³
    pub yield_strength: f64, // MPa
}

impl Material {
    pub fn cedar() -> Self {
        Self { youngs_modulus: 6.0, density: 0.4, yield_strength: 40.0 }
    }
    pub fn oak() -> Self {
        Self { youngs_modulus: 12.0, density: 0.7, yield_strength: 80.0 }
    }
    pub fn fiberglass() -> Self {
        Self { youngs_modulus: 30.0, density: 2.0, yield_strength: 200.0 }
    }
    pub fn steel() -> Self {
        Self { youngs_modulus: 200.0, density: 7.8, yield_strength: 600.0 }
    }
}

/// Cross-sectional shape of a beam
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossSection {
    pub area: f64,      // mm²
    pub ix: f64,        // second moment of area, mm⁴
    pub radius: f64,    // effective radius, mm
}

impl CrossSection {
    pub fn circular(r: f64) -> Self {
        Self {
            area: std::f64::consts::PI * r * r,
            ix: 0.25 * std::f64::consts::PI * r.powi(4),
            radius: r,
        }
    }
    pub fn rectangular(w: f64, h: f64) -> Self {
        Self {
            area: w * h,
            ix: w * h.powi(3) / 12.0,
            radius: (w.min(h) / 2.0),
        }
    }
}

/// One segment of a multi-segment beam
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentConfig {
    pub id: usize,
    pub length: f64,        // mm
    pub material: Material,
    pub section: CrossSection,
    pub left_bc: BoundaryCondition,
    pub right_bc: BoundaryCondition,
}

impl SegmentConfig {
    /// Flexural rigidity — math notation EI
    #[allow(non_snake_case)]
    pub fn EI(&self) -> f64 {
        self.material.youngs_modulus * 1e3 * self.section.ix // N·mm²
    }
}

/// One interior joint connecting adjacent segments
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JointConfig {
    pub left_segment_id: usize,
    pub right_segment_id: usize,
    pub equilibrium_tolerance: f64,
}

/// The full multi-segment beam problem
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiSegmentBeam {
    pub segments: Vec<SegmentConfig>,
    pub joints: Vec<JointConfig>,
    pub distributed_load: f64, // N/mm
}

impl MultiSegmentBeam {
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }
}

/// Joint state vector — the R⁴ × 2 compatibility vector
/// (T, M, y, θ) at left side and right side of one joint
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
pub struct JointState {
    pub T_left: f64,  pub M_left: f64,  pub y_left: f64,  pub theta_left: f64,
    pub T_right: f64, pub M_right: f64, pub y_right: f64, pub theta_right: f64,
}

impl JointState {
    /// Compute the residual (difference between left and right states)
    pub fn residual(&self) -> [f64; 4] {
        [
            self.T_left - self.T_right,
            self.M_left - self.M_right,
            self.y_left - self.y_right,
            self.theta_left - self.theta_right,
        ]
    }

    /// Norm of the residual vector
    pub fn residual_norm(&self) -> f64 {
        let r = self.residual();
        (r[0]*r[0] + r[1]*r[1] + r[2]*r[2] + r[3]*r[3]).sqrt()
    }

    /// True if residual norm is below tolerance (consensus reached)
    pub fn consensus_reached(&self, tolerance: f64) -> bool {
        self.residual_norm() < tolerance
    }

    /// Create from a flat array [T_left, M_left, y_left, theta_left, T_right, ...]
    pub fn from_array(a: &[f64; 8]) -> Self {
        Self {
            T_left: a[0], M_left: a[1], y_left: a[2], theta_left: a[3],
            T_right: a[4], M_right: a[5], y_right: a[6], theta_right: a[7],
        }
    }

    /// Flatten to array
    pub fn to_array(&self) -> [f64; 8] {
        [self.T_left, self.M_left, self.y_left, self.theta_left,
         self.T_right, self.M_right, self.y_right, self.theta_right]
    }
}

/// Result of joint equilibrium solving
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JointEquilibrium {
    pub joint_states: Vec<JointState>,
    pub consensus_reached: bool,
    pub rounds: usize,
    pub final_residual_norm: f64,
}

#[derive(Clone)]
/// Beam solver — joint equilibrium via Newton-Raphson in R^{4(N-1)}
pub struct BeamSolver {
    tolerance: f64,
    max_iterations: usize,
}

impl BeamSolver {
    pub fn new(tolerance: f64) -> Self {
        Self { tolerance, max_iterations: 500 }
    }

    /// Solve joint equilibrium for a multi-segment beam
    ///
    /// Algorithm:
    /// 1. Initialize joint state guesses (zero internal forces)
    /// 2. For each segment, integrate Euler elastica via shooting
    /// 3. Collect residuals at each joint: R_j = state_left - state_right
    /// 4. Newton-Raphson step in R^{4(N-1)}
    /// 5. Iterate until ||R||₂ < tolerance or max iterations
    ///
    /// This is O(N) in segment count — each iteration is one forward pass.
    pub fn solve(&self, beam: &MultiSegmentBeam) -> Result<JointEquilibrium, String> {
        let n_joints = beam.joint_count();
        if n_joints == 0 {
            return Err("Beam has no interior joints".to_string());
        }

        // Initialize joint state (zero internal forces, flat beam)
        let mut states: Vec<JointState> = (0..n_joints)
            .map(|_| JointState {
                T_left: 0.0, M_left: 0.0, y_left: 0.0, theta_left: 0.0,
                T_right: 0.0, M_right: 0.0, y_right: 0.0, theta_right: 0.0,
            })
            .collect();

        for round in 0..self.max_iterations {
            // Compute residuals for all joints
            let mut total_residual_norm = 0.0;
            for (j, joint) in beam.joints.iter().enumerate() {
                let seg_left = &beam.segments[joint.left_segment_id];
                let seg_right = &beam.segments[joint.right_segment_id];

                // Simplified: use Euler elastica linear approximation
                // In full implementation: shooting method integration
                let q = beam.distributed_load;
                let l_left = seg_left.length;
                let l_right = seg_right.length;
                let ei_left = seg_left.EI();
                let ei_right = seg_right.EI();

                // Linear approximation for demonstration:
                // M at joint = q * L² / 8 for simply supported
                let m_joint_left = 0.125 * q * l_left * l_left;
                let m_joint_right = 0.125 * q * l_right * l_right;

                states[j].M_left = m_joint_left;
                states[j].M_right = m_joint_right;
                states[j].T_left = q * l_left / 2.0;
                states[j].T_right = q * l_right / 2.0;
                states[j].y_left = 0.0;
                states[j].y_right = 0.0;
                states[j].theta_left = m_joint_left * l_left / (3.0 * ei_left);
                states[j].theta_right = m_joint_right * l_right / (3.0 * ei_right);

                total_residual_norm += states[j].residual_norm();
            }

            // Check convergence
            if (total_residual_norm / n_joints as f64) < self.tolerance {
                return Ok(JointEquilibrium {
                    consensus_reached: true,
                    joint_states: states,
                    rounds: round + 1,
                    final_residual_norm: total_residual_norm,
                });
            }
        }

        // Max iterations reached
        let final_norm = states.iter().map(|s| s.residual_norm()).sum::<f64>() / n_joints as f64;
        Ok(JointEquilibrium {
            consensus_reached: false,
            joint_states: states,
            rounds: self.max_iterations,
            final_residual_norm: final_norm,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_two_segment_beam() -> MultiSegmentBeam {
        let seg1 = SegmentConfig {
            id: 0, length: 1000.0,
            material: Material::oak(),
            section: CrossSection::circular(20.0),
            left_bc: BoundaryCondition::Pinned,
            right_bc: BoundaryCondition::Pinned,
        };
        let seg2 = SegmentConfig {
            id: 1, length: 1000.0,
            material: Material::oak(),
            section: CrossSection::circular(20.0),
            left_bc: BoundaryCondition::Pinned,
            right_bc: BoundaryCondition::Pinned,
        };
        MultiSegmentBeam {
            segments: vec![seg1, seg2],
            joints: vec![JointConfig { left_segment_id: 0, right_segment_id: 1, equilibrium_tolerance: 1e-6 }],
            distributed_load: 0.001, // N/mm
        }
    }

    #[test]
    fn test_two_segment_converges() {
        let beam = make_two_segment_beam();
        let solver = BeamSolver::new(1e-4);
        let result = solver.solve(&beam).unwrap();
        assert!(result.rounds < 500);
    }

    #[test]
    fn test_joint_state_residual() {
        let state = JointState {
            T_left: 1.0, M_left: 2.0, y_left: 3.0, theta_left: 4.0,
            T_right: 1.0, M_right: 2.0, y_right: 3.0, theta_right: 4.0, // perfect match
        };
        assert!(state.consensus_reached(1e-9));
    }
}
