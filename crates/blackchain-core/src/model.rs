use std::collections::HashMap;
use blackchain_crypto::Hash;
use serde::{Deserialize, Serialize};

/// Quantization format enforced for deterministic inference execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationFormat {
    /// 8-bit integer symmetric quantization
    Int8Symmetric,
    /// 4-bit integer standard GGUF format
    Int4GGUF,
    /// 16-bit brain floating point (for TEE/attested modes)
    Bfloat16,
}

/// Metadata and hardware requirements for a registered AI model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDefinition {
    /// Human-readable model identifier (e.g., "llama-3-8b-instruct")
    pub id: String,
    /// Canonical Blake3 hash of the model weights file (safetensors or gguf)
    pub weights_hash: Hash,
    /// Enforced quantization profile to ensure deterministic greedy decoding
    pub quantization: QuantizationFormat,
    /// Parameter count in billions (e.g. 8 for 8B)
    pub parameter_count_billions: u32,
    /// Compute Unit multiplier in basis points (100 = 1.0x, 850 = 8.5x)
    pub cu_multiplier_bps: u32,
    /// Minimum VRAM required in Megabytes
    pub min_vram_mb: u32,
    /// Maximum context window supported
    pub max_context_tokens: u32,
}

impl ModelDefinition {
    /// Calculate Compute Units for a given prompt and completion length.
    pub fn calculate_compute_units(&self, prompt_tokens: u32, completion_tokens: u32) -> u64 {
        // Prompt prefill weight = 1, decode token weight = 4 (decoding is memory-bandwidth bound)
        let raw_work = (prompt_tokens as u64) + ((completion_tokens as u64) * 4);
        (raw_work * (self.cu_multiplier_bps as u64)) / 100
    }
}

/// On-chain curated model registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    models: HashMap<String, ModelDefinition>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };

        // Register default reference models
        registry.register(ModelDefinition {
            id: "llama-3-8b-instruct".to_string(),
            weights_hash: Hash::ZERO, // Standard placeholder until canonical hash pinned
            quantization: QuantizationFormat::Int8Symmetric,
            parameter_count_billions: 8,
            cu_multiplier_bps: 100, // 1.0x baseline
            min_vram_mb: 12_000,
            max_context_tokens: 8192,
        });

        registry.register(ModelDefinition {
            id: "mistral-7b-instruct".to_string(),
            weights_hash: Hash::ZERO,
            quantization: QuantizationFormat::Int8Symmetric,
            parameter_count_billions: 7,
            cu_multiplier_bps: 90, // 0.9x
            min_vram_mb: 10_000,
            max_context_tokens: 8192,
        });

        registry.register(ModelDefinition {
            id: "llama-3-70b-instruct".to_string(),
            weights_hash: Hash::ZERO,
            quantization: QuantizationFormat::Int4GGUF,
            parameter_count_billions: 70,
            cu_multiplier_bps: 850, // 8.5x
            min_vram_mb: 48_000,
            max_context_tokens: 8192,
        });

        registry
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, model: ModelDefinition) {
        self.models.insert(model.id.clone(), model);
    }

    pub fn get(&self, model_id: &str) -> Option<&ModelDefinition> {
        self.models.get(model_id)
    }

    pub fn is_supported(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_units_calculation() {
        let registry = ModelRegistry::default();
        let llama8b = registry.get("llama-3-8b-instruct").unwrap();

        // 100 prompt tokens, 50 completion tokens
        // raw = 100 + (50 * 4) = 300
        // cu = (300 * 100) / 100 = 300
        let cu = llama8b.calculate_compute_units(100, 50);
        assert_eq!(cu, 300);

        let llama70b = registry.get("llama-3-70b-instruct").unwrap();
        // cu = (300 * 850) / 100 = 2550
        let cu_70b = llama70b.calculate_compute_units(100, 50);
        assert_eq!(cu_70b, 2550);
    }
}
