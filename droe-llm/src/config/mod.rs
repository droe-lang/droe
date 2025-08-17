use serde::{Deserialize, Serialize};
use crate::intelligence::InferenceMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub llm: LLMSettings,
    pub modes: ModeConfigs,
    pub safety: SafetySettings,
    pub performance: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSettings {
    pub model: String,
    pub ollama_url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfigs {
    pub regular: ModeConfig,
    pub robotics: ModeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub max_tokens: u32,
    pub system_prompt_prefix: String,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySettings {
    pub enable_validation: bool,
    pub workspace_bounds: WorkspaceBounds,
    pub max_commands: u32,
    pub require_emergency_stop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
    pub max_distance: f32,
    pub max_rotation: f32,
    pub max_wait_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub cache_responses: bool,
    pub enable_streaming: bool,
    pub max_concurrent_requests: u32,
    pub partial_update_threshold: usize,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            llm: LLMSettings {
                model: "droe-scribe:latest".to_string(),
                ollama_url: "http://localhost:11434".to_string(),
                timeout_ms: 30000,
            },
            modes: ModeConfigs {
                regular: ModeConfig {
                    temperature: 0.7,
                    top_p: 0.9,
                    top_k: 40,
                    max_tokens: 800,
                    system_prompt_prefix: "Generate DROE DSL code for general programming tasks.".to_string(),
                    stop_sequences: vec!["Human:".to_string(), "User:".to_string()],
                },
                robotics: ModeConfig {
                    temperature: 0.05,
                    top_p: 0.8,
                    top_k: 20,
                    max_tokens: 200,
                    system_prompt_prefix: "Generate ONLY robotics DROE commands. Use ultra-compact syntax.".to_string(),
                    stop_sequences: vec!["Human:".to_string(), "User:".to_string(), "\n\n".to_string()],
                },
            },
            safety: SafetySettings {
                enable_validation: true,
                workspace_bounds: WorkspaceBounds {
                    x_min: -2.0,
                    x_max: 2.0,
                    y_min: -2.0,
                    y_max: 2.0,
                    z_min: 0.0,
                    z_max: 1.5,
                    max_distance: 2.0,
                    max_rotation: 180.0,
                    max_wait_time: 10.0,
                },
                max_commands: 5,
                require_emergency_stop: true,
            },
            performance: PerformanceSettings {
                cache_responses: true,
                enable_streaming: true,
                max_concurrent_requests: 10,
                partial_update_threshold: 100, // lines
            },
        }
    }
}

impl LLMConfig {
    pub fn load_from_file(path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        
        settings.try_deserialize()
    }

    pub fn get_mode_config(&self, mode: InferenceMode) -> &ModeConfig {
        match mode {
            InferenceMode::Regular => &self.modes.regular,
            InferenceMode::Robotics => &self.modes.robotics,
        }
    }

    pub fn create_system_prompt(&self, mode: InferenceMode, base_prompt: &str) -> String {
        let mode_config = self.get_mode_config(mode);
        let safety_constraints = match mode {
            InferenceMode::Robotics => {
                format!(
                    "\n\nSAFETY CONSTRAINTS:\n\
                    - Maximum {} commands per sequence\n\
                    - Workspace bounds: x({:.1}-{:.1}), y({:.1}-{:.1}), z({:.1}-{:.1})\n\
                    - Max distance: {:.1}m, Max rotation: {:.1}°, Max wait: {:.1}s\n\
                    - ONLY use: pick, move, place, wait, scan commands",
                    self.safety.max_commands,
                    self.safety.workspace_bounds.x_min,
                    self.safety.workspace_bounds.x_max,
                    self.safety.workspace_bounds.y_min,
                    self.safety.workspace_bounds.y_max,
                    self.safety.workspace_bounds.z_min,
                    self.safety.workspace_bounds.z_max,
                    self.safety.workspace_bounds.max_distance,
                    self.safety.workspace_bounds.max_rotation,
                    self.safety.workspace_bounds.max_wait_time,
                )
            }
            InferenceMode::Regular => "".to_string(),
        };

        format!(
            "{}\n\n{}\n{}",
            mode_config.system_prompt_prefix,
            base_prompt,
            safety_constraints
        )
    }
}