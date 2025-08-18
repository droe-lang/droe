use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::{InferenceMode, config::WorkspaceBounds, LLMError};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub is_safe: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub safety: Option<SafetyValidation>,
    pub ros2: Option<ROS2Validation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyValidation {
    pub workspace_bounded: bool,
    pub collision_free: bool,
    pub emergency_stop_available: bool,
    pub safety_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROS2Validation {
    pub services_available: bool,
    pub missing_services: Vec<String>,
    pub parameters_valid: bool,
}

pub struct ValidationEngine {
    robotics_commands: HashSet<String>,
    #[allow(dead_code)]
    coordinate_pattern: Regex,
    #[allow(dead_code)]
    distance_pattern: Regex,
    #[allow(dead_code)]
    rotation_pattern: Regex,
    wait_pattern: Regex,
    pick_pattern: Regex,
    move_pattern: Regex,
    place_pattern: Regex,
    scan_pattern: Regex,
}

impl ValidationEngine {
    pub fn new() -> Self {
        let mut robotics_commands = HashSet::new();
        robotics_commands.insert("pick".to_string());
        robotics_commands.insert("move".to_string());
        robotics_commands.insert("place".to_string());
        robotics_commands.insert("wait".to_string());
        robotics_commands.insert("scan".to_string());

        Self {
            robotics_commands,
            coordinate_pattern: Regex::new(r"[xyz]:\s*(-?\d+\.?\d*)").unwrap(),
            distance_pattern: Regex::new(r"(\d+\.?\d*)\s*m?").unwrap(),
            rotation_pattern: Regex::new(r"(\d+\.?\d*)\s*[°deg]?").unwrap(),
            wait_pattern: Regex::new(r"wait\s+(\d+\.?\d*)").unwrap(),
            pick_pattern: Regex::new(r#"pick\s+"([^"]+)""#).unwrap(),
            move_pattern: Regex::new(r#"move\s+"([^"]+)"\s+(\d+\.?\d*)"#).unwrap(),
            place_pattern: Regex::new(r#"place\s+"([^"]+)""#).unwrap(),
            scan_pattern: Regex::new(r#"scan\s+"([^"]+)""#).unwrap(),
        }
    }

    pub fn validate(
        &self,
        code: &str,
        mode: InferenceMode,
        workspace_bounds: &Option<WorkspaceBounds>,
    ) -> Result<ValidationResult, LLMError> {
        match mode {
            InferenceMode::Regular => self.validate_regular_dsl(code),
            InferenceMode::Robotics => self.validate_robotics_dsl(code, workspace_bounds),
        }
    }

    pub fn validate_partial(
        &self,
        partial_code: &str,
        mode: InferenceMode,
        workspace_bounds: &Option<WorkspaceBounds>,
    ) -> ValidationResult {
        // For partial validation, we're more lenient with incomplete syntax
        match mode {
            InferenceMode::Regular => self.validate_partial_regular(partial_code),
            InferenceMode::Robotics => self.validate_partial_robotics(partial_code, workspace_bounds),
        }
    }

    fn validate_regular_dsl(&self, code: &str) -> Result<ValidationResult, LLMError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic DROE DSL syntax validation
        let lines: Vec<&str> = code.lines().collect();
        let mut in_module = false;
        let mut in_action = false;
        let mut brace_count = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Check for basic DROE keywords
            if trimmed.starts_with("module ") {
                if in_module {
                    errors.push(format!("Line {}: Nested modules not allowed", line_num + 1));
                }
                in_module = true;
            } else if trimmed.starts_with("end module") {
                if !in_module {
                    errors.push(format!("Line {}: 'end module' without matching 'module'", line_num + 1));
                }
                in_module = false;
            } else if trimmed.starts_with("action ") {
                if !in_module {
                    warnings.push(format!("Line {}: Action defined outside module", line_num + 1));
                }
                in_action = true;
            } else if trimmed.starts_with("end action") {
                if !in_action {
                    errors.push(format!("Line {}: 'end action' without matching 'action'", line_num + 1));
                }
                in_action = false;
            }

            // Count braces for structure validation
            brace_count += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_count -= trimmed.chars().filter(|&c| c == '}').count() as i32;
        }

        if brace_count != 0 {
            errors.push("Mismatched braces in code".to_string());
        }

        if in_module {
            errors.push("Unclosed module".to_string());
        }

        if in_action {
            errors.push("Unclosed action".to_string());
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            is_safe: true, // Regular DSL doesn't have safety concerns
            errors,
            warnings,
            safety: None,
            ros2: None,
        })
    }

    fn validate_robotics_dsl(
        &self,
        code: &str,
        workspace_bounds: &Option<WorkspaceBounds>,
    ) -> Result<ValidationResult, LLMError> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        let mut safety_warnings = Vec::new();

        let lines: Vec<&str> = code.lines().filter(|l| !l.trim().is_empty()).collect();
        
        // Check command count limit
        if lines.len() > 5 {
            errors.push(format!("Too many commands: {} (max 5 allowed)", lines.len()));
        }

        let mut pick_count = 0;
        let mut place_count = 0;
        let mut has_unsafe_operations = false;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Validate robotics command syntax
            if let Some(command) = self.extract_command(trimmed) {
                match command.as_str() {
                    "pick" => {
                        pick_count += 1;
                        if !self.pick_pattern.is_match(trimmed) {
                            errors.push(format!("Line {}: Invalid pick syntax", line_num + 1));
                        }
                    }
                    "move" => {
                        if let Some(caps) = self.move_pattern.captures(trimmed) {
                            if let Ok(distance) = caps[2].parse::<f32>() {
                                if let Some(bounds) = workspace_bounds {
                                    if distance > bounds.max_distance {
                                        errors.push(format!(
                                            "Line {}: Move distance {:.1}m exceeds limit {:.1}m",
                                            line_num + 1, distance, bounds.max_distance
                                        ));
                                        has_unsafe_operations = true;
                                    }
                                }
                                if distance > 3.0 {
                                    safety_warnings.push(format!(
                                        "Line {}: Large movement distance {:.1}m - verify safety",
                                        line_num + 1, distance
                                    ));
                                }
                            }
                        } else {
                            errors.push(format!("Line {}: Invalid move syntax", line_num + 1));
                        }
                    }
                    "place" => {
                        place_count += 1;
                        if !self.place_pattern.is_match(trimmed) {
                            errors.push(format!("Line {}: Invalid place syntax", line_num + 1));
                        }
                    }
                    "wait" => {
                        if let Some(caps) = self.wait_pattern.captures(trimmed) {
                            if let Ok(wait_time) = caps[1].parse::<f32>() {
                                if let Some(bounds) = workspace_bounds {
                                    if wait_time > bounds.max_wait_time {
                                        errors.push(format!(
                                            "Line {}: Wait time {:.1}s exceeds limit {:.1}s",
                                            line_num + 1, wait_time, bounds.max_wait_time
                                        ));
                                    }
                                }
                            }
                        } else {
                            errors.push(format!("Line {}: Invalid wait syntax", line_num + 1));
                        }
                    }
                    "scan" => {
                        if !self.scan_pattern.is_match(trimmed) {
                            errors.push(format!("Line {}: Invalid scan syntax", line_num + 1));
                        }
                    }
                    _ => {
                        errors.push(format!("Line {}: Unknown robotics command '{}'", line_num + 1, command));
                    }
                }
            } else {
                errors.push(format!("Line {}: Invalid command syntax", line_num + 1));
            }
        }

        // Validate pick/place pairing
        if pick_count != place_count {
            safety_warnings.push(format!(
                "Unbalanced pick/place operations: {} picks, {} places",
                pick_count, place_count
            ));
        }

        // Check for emergency stop availability (simplified check)
        let has_emergency_stop = code.contains("emergency") || code.contains("stop");

        let safety_validation = SafetyValidation {
            workspace_bounded: workspace_bounds.is_some(),
            collision_free: !has_unsafe_operations,
            emergency_stop_available: has_emergency_stop,
            safety_warnings: safety_warnings.clone(),
        };

        // Mock ROS2 validation (would connect to actual ROS2 services in production)
        let ros2_validation = ROS2Validation {
            services_available: true, // Would check actual services
            missing_services: vec![], // Would populate with missing services
            parameters_valid: errors.is_empty(),
        };

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            is_safe: !has_unsafe_operations && pick_count == place_count,
            errors,
            warnings,
            safety: Some(safety_validation),
            ros2: Some(ros2_validation),
        })
    }

    fn validate_partial_regular(&self, partial_code: &str) -> ValidationResult {
        // For partial regular code, just check basic syntax without requiring completeness
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Very basic validation for partial code
        if partial_code.trim().is_empty() {
            return ValidationResult {
                is_valid: true,
                is_safe: true,
                errors,
                warnings,
                safety: None,
                ros2: None,
            };
        }

        // Check for obvious syntax errors in partial code
        let brace_balance = partial_code.chars().filter(|&c| c == '{').count() as i32 
                          - partial_code.chars().filter(|&c| c == '}').count() as i32;
        
        if brace_balance < 0 {
            errors.push("Too many closing braces".to_string());
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            is_safe: true,
            errors,
            warnings,
            safety: None,
            ros2: None,
        }
    }

    fn validate_partial_robotics(&self, partial_code: &str, _workspace_bounds: &Option<WorkspaceBounds>) -> ValidationResult {
        // For partial robotics code, validate what we can without requiring completeness
        let errors = Vec::new();
        let mut warnings = Vec::new();

        if partial_code.trim().is_empty() {
            return ValidationResult {
                is_valid: true,
                is_safe: true,
                errors,
                warnings,
                safety: None,
                ros2: None,
            };
        }

        // Check if partial code looks like it's going in the right direction
        let lines: Vec<&str> = partial_code.lines().filter(|l| !l.trim().is_empty()).collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Check if this looks like a robotics command
            if let Some(command) = self.extract_partial_command(trimmed) {
                if !self.robotics_commands.contains(&command) {
                    warnings.push(format!("Line {}: '{}' is not a recognized robotics command", line_num + 1, command));
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            is_safe: warnings.is_empty(),
            errors,
            warnings,
            safety: None,
            ros2: None,
        }
    }

    fn extract_command(&self, line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        if !words.is_empty() {
            Some(words[0].to_string())
        } else {
            None
        }
    }

    fn extract_partial_command(&self, line: &str) -> Option<String> {
        // More lenient extraction for partial commands
        let words: Vec<&str> = line.split_whitespace().collect();
        if !words.is_empty() {
            Some(words[0].to_string())
        } else {
            None
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}