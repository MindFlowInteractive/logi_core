use crate::models::PlayerPerformance;

/// Difficulty scaler with adjustable weights and smoothing.
#[derive(Debug, Clone)]
pub struct DifficultyScaler {
    /// Current difficulty in [0.0, 1.0]
    pub difficulty: f32,
    /// Minimum allowed difficulty
    pub min_difficulty: f32,
    /// Maximum allowed difficulty
    pub max_difficulty: f32,
    /// Weights for the three components (time, accuracy, streak)
    pub time_weight: f32,
    pub accuracy_weight: f32,
    pub streak_weight: f32,
    /// smoothing factor to avoid abrupt jumps (0.0 - 1.0)
    pub smoothing: f32,
}

impl Default for DifficultyScaler {
    fn default() -> Self {
        Self {
            difficulty: 0.5,
            min_difficulty: 0.05,
            max_difficulty: 1.0,
            time_weight: 0.45,
            accuracy_weight: 0.35,
            streak_weight: 0.20,
            smoothing: 0.2,
        }
    }
}

impl DifficultyScaler {
    /// Create a new scaler with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fully customizable constructor.
    pub fn with_params(
        difficulty: f32,
        min_difficulty: f32,
        max_difficulty: f32,
        time_weight: f32,
        accuracy_weight: f32,
        streak_weight: f32,
        smoothing: f32,
    ) -> Self {
        Self {
            difficulty,
            min_difficulty,
            max_difficulty,
            time_weight,
            accuracy_weight,
            streak_weight,
            smoothing,
        }
    }

    /// Update difficulty using a PlayerPerformance snapshot.
    /// Returns the new difficulty.
    pub fn update_difficulty(&mut self, perf: &PlayerPerformance) -> f32 {
        let mut p = perf.clone();
        // sanitize inputs
        p.sanitize();

        // --- Compute sub-factors ---
        // Time factor: ratio target_time / completion_time
        // If player is faster than target, ratio > 1 => positive
        let mut time_ratio = p.target_time / p.completion_time;
        // Prevent extreme ratios
        time_ratio = time_ratio.clamp(0.5, 1.5);
        // Convert to centered range [-0.5, +0.5] approximately
        let time_factor = time_ratio - 1.0; // => [-0.5, +0.5]

        // Accuracy factor: center around baseline (0.7)
        let baseline = 0.7_f32;
        let mut accuracy_delta = p.accuracy_rate - baseline;
        accuracy_delta = accuracy_delta.clamp(-0.4, 0.4); // avoid huge swings
        // normalize to approx [-0.4, +0.4]

        // Streak factor: smooth growth with log to avoid runaway
        let streak_factor = (p.consecutive_correct as f32).ln_1p() / 4.0; // scaled small

        // --- Weighted adjustment ---
        let adj = time_factor * self.time_weight
            + accuracy_delta * self.accuracy_weight
            + streak_factor * self.streak_weight;

        // smoothing - move current difficulty towards target by smoothing factor
        let proposed = (self.difficulty + adj * self.smoothing).clamp(self.min_difficulty, self.max_difficulty);

        self.difficulty = proposed;
        self.difficulty
    }

    /// Directly set difficulty (clamped)
    pub fn set_difficulty(&mut self, val: f32) {
        self.difficulty = val.clamp(self.min_difficulty, self.max_difficulty);
    }
}
