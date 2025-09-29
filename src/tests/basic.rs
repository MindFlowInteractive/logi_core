use puzzle_difficulty::{DifficultyScaler, PlayerPerformance};

#[test]
fn increases_on_good_perf() {
    let mut s = DifficultyScaler::new();
    let before = s.difficulty;

    let perf = PlayerPerformance {
        completion_time: 10.0,
        target_time: 20.0,
        accuracy_rate: 0.95,
        consecutive_correct: 6,
    };

    let _ = s.update_difficulty(&perf);
    assert!(s.difficulty > before, "Expected difficulty to increase on strong perf");
}

#[test]
fn decreases_on_bad_perf() {
    let mut s = DifficultyScaler::new();
    s.set_difficulty(0.6);

    let perf = PlayerPerformance {
        completion_time: 40.0,   // very slow
        target_time: 20.0,
        accuracy_rate: 0.2,      // low accuracy
        consecutive_correct: 0,
    };

    let _ = s.update_difficulty(&perf);
    assert!(s.difficulty < 0.6, "Expected difficulty to lower for poor perf");
}
