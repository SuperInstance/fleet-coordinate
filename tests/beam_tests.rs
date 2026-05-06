use fleet_coordinate::beam::{BeamSolver, MultiSegmentBeam, SegmentConfig, JointConfig, Material, CrossSection, BoundaryCondition};

fn make_two_seg() -> MultiSegmentBeam {
    MultiSegmentBeam {
        segments: vec![
            SegmentConfig {
                id: 0, length: 1000.0,
                material: Material::oak(),
                section: CrossSection::circular(20.0),
                left_bc: BoundaryCondition::Pinned,
                right_bc: BoundaryCondition::Pinned,
            },
            SegmentConfig {
                id: 1, length: 1000.0,
                material: Material::oak(),
                section: CrossSection::circular(20.0),
                left_bc: BoundaryCondition::Pinned,
                right_bc: BoundaryCondition::Pinned,
            },
        ],
        joints: vec![JointConfig { left_segment_id: 0, right_segment_id: 1, equilibrium_tolerance: 1e-6 }],
        distributed_load: 0.001,
    }
}

#[test]
fn test_two_segment_solver() {
    let beam = make_two_seg();
    let solver = BeamSolver::new(1e-4);
    let result = solver.solve(&beam).unwrap();
    assert!(result.rounds > 0);
}
