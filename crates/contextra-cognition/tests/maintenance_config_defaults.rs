use contextra_cognition::maintenance_config::MaintenanceConfig;

#[test]
fn test_maintenance_config_defaults() {
    let config = MaintenanceConfig::default();

    assert_eq!(config.tick_interval_secs, 60);
    assert_eq!(config.decay_enabled, true);
    assert_eq!(config.percolation_enabled, true);
    assert_eq!(config.replicator_enabled, false);
    assert!((config.replicator_lr - 0.05).abs() < f32::EPSILON);
    assert!((config.coherence_bonus_beta - 0.15).abs() < f32::EPSILON);
    assert_eq!(config.background_consolidation_enabled, false);
    assert_eq!(config.background_consolidation_episode_threshold, 50);
}
