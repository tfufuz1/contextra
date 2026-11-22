//! Differential test for `CatoniDriftDetector` vs Naive Arithmetic Mean Detector.
//!
//! Verifies that `CatoniDriftDetector` does NOT falsely report drift when single extreme reward
//! outliers are injected into an otherwise stable reward stream, whereas a naive arithmetic mean
//! detector falsely triggers drift. Also verifies that `CatoniDriftDetector` correctly detects
//! true sustained distribution shifts.

use contextra_adapt::{CatoniDriftDetector, DriftDetector, DriftSignal};

/// Naive reference detector using moving arithmetic mean without robust M-estimation.
struct NaiveMeanDriftDetector {
    window: Vec<f32>,
    window_size: usize,
    baseline_mean: f32,
    threshold: f32,
}

impl NaiveMeanDriftDetector {
    fn new(baseline_mean: f32, threshold: f32, window_size: usize) -> Self {
        Self {
            window: Vec::with_capacity(window_size),
            window_size,
            baseline_mean,
            threshold,
        }
    }

    fn observe(&mut self, reward: f32) -> DriftSignal {
        self.window.push(reward);
        if self.window.len() > self.window_size {
            self.window.remove(0);
        }
        let current_mean: f32 = self.window.iter().sum::<f32>() / self.window.len() as f32;
        if (current_mean - self.baseline_mean).abs() >= self.threshold {
            DriftSignal::Detected
        } else {
            DriftSignal::Stable
        }
    }
}

#[test]
fn test_catoni_robust_to_extreme_outliers_compared_to_naive_mean() {
    let baseline_reward = 0.5f64;
    let threshold = 0.5f64;
    let alpha = 0.05f64;

    let mut catoni = CatoniDriftDetector::new(baseline_reward, threshold, alpha);
    let mut naive = NaiveMeanDriftDetector::new(0.5, 0.2, 10);

    // Warmup phase with stable rewards
    for _ in 0..50 {
        let sig_c = catoni.observe(0.5);
        let sig_n = naive.observe(0.5);
        assert_eq!(sig_c, DriftSignal::Stable);
        assert_eq!(sig_n, DriftSignal::Stable);
    }

    // Inject a single extreme reward outlier (e.g., reward = 100.0)
    let outlier = 100.0f32;
    let catoni_signal_on_outlier = catoni.observe(outlier);
    let naive_signal_on_outlier = naive.observe(outlier);

    // Assert differential behavior: Catoni remains stable, Naive triggers false positive drift
    assert_eq!(
        catoni_signal_on_outlier,
        DriftSignal::Stable,
        "CatoniDriftDetector must remain Stable on isolated outlier"
    );
    assert_eq!(
        naive_signal_on_outlier,
        DriftSignal::Detected,
        "Naive arithmetic mean detector falsely triggers Detected on outlier"
    );
    assert!(
        !catoni.is_drifting(),
        "CatoniDriftDetector::is_drifting() must return false"
    );

    // Return to normal stream and ensure Catoni stays stable
    for _ in 0..20 {
        let sig_c = catoni.observe(0.5);
        assert_eq!(sig_c, DriftSignal::Stable);
    }
}

#[test]
fn test_catoni_detects_true_sustained_distribution_shift() {
    let baseline_reward = 0.5f64;
    let threshold = 0.5f64;
    let alpha = 0.05f64;

    let mut catoni = CatoniDriftDetector::new(baseline_reward, threshold, alpha);

    // Baseline stream
    for _ in 0..50 {
        catoni.observe(0.5);
    }
    assert!(!catoni.is_drifting());

    // Sustained distribution shift: rewards change permanently from 0.5 to 0.95
    let mut detected = false;
    for _ in 0..50 {
        if catoni.observe(0.95) == DriftSignal::Detected {
            detected = true;
            break;
        }
    }

    assert!(
        detected,
        "CatoniDriftDetector must detect sustained true distribution shift"
    );
    assert!(catoni.is_drifting());
}
