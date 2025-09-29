use serde::{Deserialize, Serialize};

/// Player performance metrics that drive difficulty adjustments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPerformance {
    /// Completion time in seconds (player's last attempt)
    pub completion_time: f32,
    /// Expected/target completion time for the puzzle (seconds)
    pub target_time: f32,
    /// Accuracy rate between 0.0 and 1.0
    pub accuracy_rate: f32,
    /// Number of consecutive correct answers (streak)
    pub consecutive_correct: u32,
}

impl PlayerPerformance {
    /// Simple validator to keep values sane.
    pub fn sanitize(&mut self) {
        if self.completion_time <= 0.0 {
            self.completion_time = 1.0;
        }
        if self.target_time <= 0.0 {
            self.target_time = 1.0;
        }
        if self.accuracy_rate.is_nan() || self.accuracy_rate < 0.0 {
            self.accuracy_rate = 0.0;
        } else if self.accuracy_rate > 1.0 {
            self.accuracy_rate = 1.0;
        }
    }
}
