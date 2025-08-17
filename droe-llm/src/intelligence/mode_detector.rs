use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InferenceMode {
    Regular,
    Robotics,
}

#[derive(Debug, Clone)]
pub struct ModeResult {
    pub mode: InferenceMode,
    pub confidence: f32,
    pub detected_keywords: Vec<String>,
}

pub struct ModeDetector {
    robotics_keywords: Vec<String>,
    robotics_patterns: Vec<Regex>,
    keyword_weights: HashMap<String, f32>,
}

impl ModeDetector {
    pub fn new() -> Self {
        let robotics_keywords = vec![
            "pick".to_string(),
            "move".to_string(),
            "place".to_string(),
            "scan".to_string(),
            "robot".to_string(),
            "gripper".to_string(),
            "manipulator".to_string(),
            "conveyor".to_string(),
            "workspace".to_string(),
            "ros2".to_string(),
            "motion".to_string(),
            "trajectory".to_string(),
            "position".to_string(),
            "orientation".to_string(),
            "collision".to_string(),
            "safety".to_string(),
            "emergency".to_string(),
            "stop".to_string(),
            "bin".to_string(),
            "object".to_string(),
            "vision".to_string(),
            "camera".to_string(),
            "sensor".to_string(),
        ];

        let robotics_patterns = vec![
            Regex::new(r#"pick\s+["']\w+["']"#).unwrap(),
            Regex::new(r#"move\s+["']\w+["']\s+\d+\.?\d*"#).unwrap(),
            Regex::new(r#"place\s+["']\w+["']"#).unwrap(),
            Regex::new(r#"wait\s+\d+\.?\d*"#).unwrap(),
            Regex::new(r#"scan\s+["']\w+["']"#).unwrap(),
            Regex::new(r"\b(x|y|z):\s*\d+\.?\d*").unwrap(),
            Regex::new(r"distance\s*[=:]\s*\d+\.?\d*").unwrap(),
            Regex::new(r"rotation\s*[=:]\s*\d+\.?\d*").unwrap(),
        ];

        let mut keyword_weights = HashMap::new();
        // High-confidence robotics keywords
        keyword_weights.insert("pick".to_string(), 0.9);
        keyword_weights.insert("place".to_string(), 0.9);
        keyword_weights.insert("robot".to_string(), 0.8);
        keyword_weights.insert("gripper".to_string(), 0.9);
        keyword_weights.insert("manipulator".to_string(), 0.9);
        keyword_weights.insert("ros2".to_string(), 0.95);
        
        // Medium-confidence robotics keywords
        keyword_weights.insert("move".to_string(), 0.6);
        keyword_weights.insert("scan".to_string(), 0.7);
        keyword_weights.insert("workspace".to_string(), 0.7);
        keyword_weights.insert("conveyor".to_string(), 0.8);
        keyword_weights.insert("safety".to_string(), 0.6);
        keyword_weights.insert("emergency".to_string(), 0.7);
        keyword_weights.insert("collision".to_string(), 0.8);
        
        // Lower-confidence robotics keywords
        keyword_weights.insert("position".to_string(), 0.4);
        keyword_weights.insert("motion".to_string(), 0.5);
        keyword_weights.insert("trajectory".to_string(), 0.7);
        keyword_weights.insert("orientation".to_string(), 0.5);
        keyword_weights.insert("object".to_string(), 0.3);
        keyword_weights.insert("vision".to_string(), 0.5);
        keyword_weights.insert("camera".to_string(), 0.4);
        keyword_weights.insert("sensor".to_string(), 0.4);
        keyword_weights.insert("bin".to_string(), 0.6);
        keyword_weights.insert("stop".to_string(), 0.3);

        Self {
            robotics_keywords,
            robotics_patterns,
            keyword_weights,
        }
    }

    pub fn detect_mode(&self, prompt: &str, context: &Option<String>) -> ModeResult {
        let combined_text = format!(
            "{} {}",
            prompt,
            context.as_deref().unwrap_or("")
        ).to_lowercase();

        let mut detected_keywords = Vec::new();
        let mut confidence_score = 0.0;
        let mut total_matches = 0;

        // Check for robotics keywords
        for keyword in &self.robotics_keywords {
            if combined_text.contains(keyword) {
                detected_keywords.push(keyword.clone());
                let weight = self.keyword_weights.get(keyword).unwrap_or(&0.5);
                confidence_score += weight;
                total_matches += 1;
            }
        }

        // Check for robotics patterns (higher weight)
        for pattern in &self.robotics_patterns {
            if pattern.is_match(&combined_text) {
                confidence_score += 0.8; // Pattern matches are high confidence
                total_matches += 1;
            }
        }

        // Check file extension and context clues
        if let Some(ctx) = context {
            if ctx.contains(".robot") || ctx.contains("robotics") || ctx.contains("ros") {
                confidence_score += 0.6;
                total_matches += 1;
            }
        }

        // Normalize confidence score
        let normalized_confidence = if total_matches > 0 {
            (confidence_score / total_matches as f32).min(1.0)
        } else {
            0.0
        };

        // Determine mode based on confidence threshold
        let mode = if normalized_confidence >= 0.3 {
            InferenceMode::Robotics
        } else {
            InferenceMode::Regular
        };

        // For robotics mode, ensure confidence is at least 0.3, for regular mode cap at 0.7
        let final_confidence = match mode {
            InferenceMode::Robotics => normalized_confidence.max(0.3),
            InferenceMode::Regular => (1.0 - normalized_confidence).min(0.9),
        };

        ModeResult {
            mode,
            confidence: final_confidence,
            detected_keywords,
        }
    }

    pub fn get_mode_hint(&self, file_path: &Option<String>, content: &Option<String>) -> Option<InferenceMode> {
        if let Some(path) = file_path {
            if path.contains("robot") || path.contains("ros") || path.ends_with(".robot") {
                return Some(InferenceMode::Robotics);
            }
        }

        if let Some(content) = content {
            if content.lines().take(10).any(|line| {
                line.contains("robotics") || line.contains("ROS") || line.contains("robot")
            }) {
                return Some(InferenceMode::Robotics);
            }
        }

        None
    }
}

impl Default for ModeDetector {
    fn default() -> Self {
        Self::new()
    }
}