use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::LLMError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaType {
    Insert,
    Delete,
    Modify,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub delta_type: DeltaType,
    pub line_number: u32,
    pub old_content: String,
    pub new_content: String,
    pub context_start: u32,
    pub context_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub lines_added: u32,
    pub lines_removed: u32,
    pub lines_modified: u32,
    pub total_lines: u32,
}

#[derive(Debug, Clone)]
pub struct PartialUpdateResult {
    pub updated_content: String,
    pub deltas: Vec<FileDelta>,
    pub stats: DiffStats,
    pub preview: String,
}

pub struct PartialUpdateEngine {
    #[allow(dead_code)]
    context_lines: usize,
}

impl PartialUpdateEngine {
    pub fn new() -> Self {
        Self {
            context_lines: 3,
        }
    }

    pub fn apply_llm_update(
        &self,
        original_content: &str,
        llm_generated_code: &str,
        prompt: &str,
    ) -> Result<PartialUpdateResult, LLMError> {
        // Determine update strategy based on prompt and content
        let update_strategy = self.determine_update_strategy(prompt, original_content, llm_generated_code);
        
        match update_strategy {
            UpdateStrategy::FullReplace => self.full_replace_update(original_content, llm_generated_code),
            UpdateStrategy::SmartMerge => self.smart_merge_update(original_content, llm_generated_code, prompt),
            UpdateStrategy::IncrementalInsert => self.incremental_insert_update(original_content, llm_generated_code, prompt),
            UpdateStrategy::FunctionReplace => self.function_replace_update(original_content, llm_generated_code, prompt),
        }
    }

    fn determine_update_strategy(&self, prompt: &str, original: &str, generated: &str) -> UpdateStrategy {
        let prompt_lower = prompt.to_lowercase();
        
        // Check for specific update keywords in prompt
        if prompt_lower.contains("replace") || prompt_lower.contains("rewrite") {
            return UpdateStrategy::FullReplace;
        }
        
        if prompt_lower.contains("add") || prompt_lower.contains("insert") || prompt_lower.contains("new") {
            return UpdateStrategy::IncrementalInsert;
        }
        
        if prompt_lower.contains("function") || prompt_lower.contains("method") || prompt_lower.contains("action") {
            return UpdateStrategy::FunctionReplace;
        }
        
        // Analyze content similarity to determine strategy
        let similarity = self.calculate_similarity(original, generated);
        
        if similarity < 0.3 {
            UpdateStrategy::FullReplace
        } else if similarity > 0.8 {
            UpdateStrategy::IncrementalInsert
        } else {
            UpdateStrategy::SmartMerge
        }
    }

    fn full_replace_update(&self, original: &str, generated: &str) -> Result<PartialUpdateResult, LLMError> {
        let diff_engine = DiffEngine::new();
        let deltas = diff_engine.compute_diff(original, generated)?;
        let stats = self.calculate_stats(&deltas);
        let preview = self.generate_preview(generated, &deltas);

        Ok(PartialUpdateResult {
            updated_content: generated.to_string(),
            deltas,
            stats,
            preview,
        })
    }

    fn smart_merge_update(&self, original: &str, generated: &str, prompt: &str) -> Result<PartialUpdateResult, LLMError> {
        // Try to intelligently merge the generated code with the original
        let original_lines: Vec<&str> = original.lines().collect();
        let generated_lines: Vec<&str> = generated.lines().collect();
        
        // Find the best insertion point based on context
        let insertion_point = self.find_best_insertion_point(&original_lines, &generated_lines, prompt);
        
        let mut merged_lines = Vec::new();
        
        // Add original content up to insertion point
        for (i, line) in original_lines.iter().enumerate() {
            if i == insertion_point {
                // Insert generated content
                for gen_line in &generated_lines {
                    merged_lines.push(gen_line.to_string());
                }
            }
            merged_lines.push(line.to_string());
        }
        
        let merged_content = merged_lines.join("\n");
        let diff_engine = DiffEngine::new();
        let deltas = diff_engine.compute_diff(original, &merged_content)?;
        let stats = self.calculate_stats(&deltas);
        let preview = self.generate_preview(&merged_content, &deltas);

        Ok(PartialUpdateResult {
            updated_content: merged_content,
            deltas,
            stats,
            preview,
        })
    }

    fn incremental_insert_update(&self, original: &str, generated: &str, prompt: &str) -> Result<PartialUpdateResult, LLMError> {
        // Insert generated content at appropriate location
        let original_lines: Vec<&str> = original.lines().collect();
        let generated_lines: Vec<&str> = generated.lines().collect();
        
        // Find insertion point based on keywords in prompt
        let insertion_point = self.find_insertion_point_from_prompt(&original_lines, prompt);
        
        let mut updated_lines = Vec::new();
        
        for (i, line) in original_lines.iter().enumerate() {
            updated_lines.push(line.to_string());
            
            if i == insertion_point {
                // Add empty line before insertion
                updated_lines.push("".to_string());
                
                // Insert generated content
                for gen_line in &generated_lines {
                    updated_lines.push(gen_line.to_string());
                }
                
                // Add empty line after insertion
                updated_lines.push("".to_string());
            }
        }
        
        let updated_content = updated_lines.join("\n");
        let diff_engine = DiffEngine::new();
        let deltas = diff_engine.compute_diff(original, &updated_content)?;
        let stats = self.calculate_stats(&deltas);
        let preview = self.generate_preview(&updated_content, &deltas);

        Ok(PartialUpdateResult {
            updated_content,
            deltas,
            stats,
            preview,
        })
    }

    fn function_replace_update(&self, original: &str, generated: &str, prompt: &str) -> Result<PartialUpdateResult, LLMError> {
        // Replace specific function/action based on prompt analysis
        let function_name = self.extract_function_name_from_prompt(prompt);
        
        if let Some(name) = function_name {
            let updated_content = self.replace_function_in_content(original, generated, &name)?;
            let diff_engine = DiffEngine::new();
            let deltas = diff_engine.compute_diff(original, &updated_content)?;
            let stats = self.calculate_stats(&deltas);
            let preview = self.generate_preview(&updated_content, &deltas);

            Ok(PartialUpdateResult {
                updated_content,
                deltas,
                stats,
                preview,
            })
        } else {
            // Fallback to smart merge if we can't identify the function
            self.smart_merge_update(original, generated, prompt)
        }
    }

    fn find_best_insertion_point(&self, original_lines: &[&str], _generated_lines: &[&str], _prompt: &str) -> usize {
        // Simple heuristic: look for module or action boundaries
        for (i, line) in original_lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("end module") || trimmed.starts_with("end action") {
                return i;
            }
        }
        
        // Default to end of file
        original_lines.len()
    }

    fn find_insertion_point_from_prompt(&self, original_lines: &[&str], prompt: &str) -> usize {
        let prompt_lower = prompt.to_lowercase();
        
        // Look for specific insertion hints in prompt
        if prompt_lower.contains("at the end") || prompt_lower.contains("after") {
            return original_lines.len();
        }
        
        if prompt_lower.contains("at the beginning") || prompt_lower.contains("before") {
            return 0;
        }
        
        // Look for contextual clues
        for (i, line) in original_lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            if prompt_lower.contains("action") && line_lower.contains("action") {
                return i + 1;
            }
            if prompt_lower.contains("module") && line_lower.contains("module") {
                return i + 1;
            }
        }
        
        // Default to middle of file
        original_lines.len() / 2
    }

    fn extract_function_name_from_prompt(&self, prompt: &str) -> Option<String> {
        // Use regex to extract function/action names from prompt
        let function_pattern = Regex::new(r"(?:function|action|method)\s+(\w+)").unwrap();
        
        if let Some(caps) = function_pattern.captures(prompt) {
            return Some(caps[1].to_string());
        }
        
        None
    }

    fn replace_function_in_content(&self, original: &str, generated: &str, function_name: &str) -> Result<String, LLMError> {
        let function_pattern = Regex::new(&format!(r"action\s+{}\s+.*?end\s+action", regex::escape(function_name))).unwrap();
        
        if function_pattern.is_match(original) {
            Ok(function_pattern.replace(original, generated).to_string())
        } else {
            // Function not found, append new function
            Ok(format!("{}\n\n{}", original, generated))
        }
    }

    fn calculate_similarity(&self, text1: &str, text2: &str) -> f32 {
        // Simple similarity calculation based on common lines
        let lines1: Vec<&str> = text1.lines().collect();
        let lines2: Vec<&str> = text2.lines().collect();
        
        let mut common_lines = 0;
        let total_lines = lines1.len().max(lines2.len());
        
        if total_lines == 0 {
            return 1.0;
        }
        
        for line1 in &lines1 {
            if lines2.contains(line1) {
                common_lines += 1;
            }
        }
        
        common_lines as f32 / total_lines as f32
    }

    fn calculate_stats(&self, deltas: &[FileDelta]) -> DiffStats {
        let mut stats = DiffStats {
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
            total_lines: 0,
        };

        for delta in deltas {
            match delta.delta_type {
                DeltaType::Insert => stats.lines_added += 1,
                DeltaType::Delete => stats.lines_removed += 1,
                DeltaType::Modify => stats.lines_modified += 1,
                DeltaType::Context => {} // Don't count context lines
            }
        }

        stats.total_lines = stats.lines_added + stats.lines_removed + stats.lines_modified;
        stats
    }

    fn generate_preview(&self, content: &str, _deltas: &[FileDelta]) -> String {
        // Generate a preview showing key changes
        let lines: Vec<&str> = content.lines().collect();
        let mut preview_lines = Vec::new();
        
        // Show first few lines
        for (i, line) in lines.iter().take(5).enumerate() {
            preview_lines.push(format!("{:3}: {}", i + 1, line));
        }
        
        if lines.len() > 10 {
            preview_lines.push("...".to_string());
            
            // Show last few lines
            for (i, line) in lines.iter().skip(lines.len() - 5).enumerate() {
                preview_lines.push(format!("{:3}: {}", lines.len() - 5 + i + 1, line));
            }
        }
        
        preview_lines.join("\n")
    }
}

#[derive(Debug, Clone)]
enum UpdateStrategy {
    FullReplace,
    SmartMerge,
    IncrementalInsert,
    FunctionReplace,
}

pub struct DiffEngine;

impl DiffEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_diff(&self, original: &str, updated: &str) -> Result<Vec<FileDelta>, LLMError> {
        let original_lines: Vec<&str> = original.lines().collect();
        let updated_lines: Vec<&str> = updated.lines().collect();
        
        let mut deltas = Vec::new();
        let mut original_idx = 0;
        let mut updated_idx = 0;
        
        while original_idx < original_lines.len() || updated_idx < updated_lines.len() {
            if original_idx >= original_lines.len() {
                // Remaining lines are insertions
                while updated_idx < updated_lines.len() {
                    deltas.push(FileDelta {
                        delta_type: DeltaType::Insert,
                        line_number: (updated_idx + 1) as u32,
                        old_content: "".to_string(),
                        new_content: updated_lines[updated_idx].to_string(),
                        context_start: updated_idx.saturating_sub(2) as u32,
                        context_end: (updated_idx + 3).min(updated_lines.len()) as u32,
                    });
                    updated_idx += 1;
                }
                break;
            }
            
            if updated_idx >= updated_lines.len() {
                // Remaining lines are deletions
                while original_idx < original_lines.len() {
                    deltas.push(FileDelta {
                        delta_type: DeltaType::Delete,
                        line_number: (original_idx + 1) as u32,
                        old_content: original_lines[original_idx].to_string(),
                        new_content: "".to_string(),
                        context_start: original_idx.saturating_sub(2) as u32,
                        context_end: (original_idx + 3).min(original_lines.len()) as u32,
                    });
                    original_idx += 1;
                }
                break;
            }
            
            if original_lines[original_idx] == updated_lines[updated_idx] {
                // Lines are the same, move to next
                original_idx += 1;
                updated_idx += 1;
            } else {
                // Lines differ, record as modification
                deltas.push(FileDelta {
                    delta_type: DeltaType::Modify,
                    line_number: (original_idx + 1) as u32,
                    old_content: original_lines[original_idx].to_string(),
                    new_content: updated_lines[updated_idx].to_string(),
                    context_start: original_idx.saturating_sub(2) as u32,
                    context_end: (original_idx + 3).min(original_lines.len()) as u32,
                });
                original_idx += 1;
                updated_idx += 1;
            }
        }
        
        Ok(deltas)
    }

    pub fn generate_unified_diff(&self, original: &str, updated: &str, file_path: &str) -> String {
        let original_lines: Vec<&str> = original.lines().collect();
        let updated_lines: Vec<&str> = updated.lines().collect();
        
        let mut diff_lines = Vec::new();
        diff_lines.push(format!("--- {}", file_path));
        diff_lines.push(format!("+++ {}", file_path));
        
        // Simple unified diff generation (could be enhanced with proper diff algorithm)
        for (i, (orig_line, upd_line)) in original_lines.iter().zip(updated_lines.iter()).enumerate() {
            if orig_line != upd_line {
                diff_lines.push(format!("@@ -{},{} +{},{} @@", i + 1, 1, i + 1, 1));
                diff_lines.push(format!("-{}", orig_line));
                diff_lines.push(format!("+{}", upd_line));
            }
        }
        
        // Handle length differences
        if original_lines.len() != updated_lines.len() {
            let min_len = original_lines.len().min(updated_lines.len());
            if original_lines.len() > min_len {
                for (i, line) in original_lines.iter().skip(min_len).enumerate() {
                    diff_lines.push(format!("@@ -{},{} +{},{} @@", min_len + i + 1, 1, min_len + i + 1, 0));
                    diff_lines.push(format!("-{}", line));
                }
            } else if updated_lines.len() > min_len {
                for (i, line) in updated_lines.iter().skip(min_len).enumerate() {
                    diff_lines.push(format!("@@ -{},{} +{},{} @@", min_len + i + 1, 0, min_len + i + 1, 1));
                    diff_lines.push(format!("+{}", line));
                }
            }
        }
        
        diff_lines.join("\n")
    }
}

impl Default for PartialUpdateEngine {
    fn default() -> Self {
        Self::new()
    }
}