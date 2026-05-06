use fleet_coordinate::{FleetCoordinate, Config};

#[test]
fn test_three_agent_triangle_self_coordinating() {
    let mut fleet = FleetCoordinate::new(Config::default());
    fleet.add_agent(1, [0.0, 0.0], vec![]);
    fleet.add_agent(2, [1.0, 0.0], vec![]);
    fleet.add_agent(3, [0.5, 0.87], vec![]);
    fleet.add_trust_edge(1, 2);
    fleet.add_trust_edge(2, 3);
    fleet.add_trust_edge(3, 1);
    let report = fleet.analyze();
    assert!(report.fleet_theorem.is_laman_rigid);
}

#[test]
fn test_trust_topology_bits() {
    let fleet = FleetCoordinate::new(Config::default());
    let v = fleet.trust_encoder().encode(0.6, 0.8);
    let (x, y) = fleet.trust_encoder().decode(v);
    assert!((x - 0.6).abs() < 0.01);
    assert!((y - 0.8).abs() < 0.01);
}
