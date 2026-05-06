use fleet_coordinate::zhc::{ZhcConsensus, ConsensusResult};

#[test]
fn test_zhc_three_agent_rigid() {
    let mut zhc = ZhcConsensus::new(0.5);
    zhc.add_tile(1, [1.0, 0.0, 0.0], vec![2, 3]);
    zhc.add_tile(2, [1.0, 0.0, 0.0], vec![1, 3]);
    zhc.add_tile(3, [1.0, 0.0, 0.0], vec![1, 2]);
    let result = zhc.run_consensus();
    assert!(result.is_consistent);
}

#[test]
fn test_zhc_information_bits() {
    let mut zhc = ZhcConsensus::new(0.5);
    for i in 1..=4 {
        zhc.add_tile(i, [1.0, 0.0, 0.0], vec![]);
    }
    let result = zhc.run_consensus();
    assert!(result.information_bits > 0.0);
}
