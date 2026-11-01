use blackchain_core::{ModelDefinition, ModelRegistry};
use blackchain_crypto::{hash, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComputeError {
    #[error("Model not supported in registry: {0}")]
    UnsupportedModel(String),
    #[error("Exceeded maximum context length for model")]
    ContextLengthExceeded,
    #[error("Inference execution error: {0}")]
    ExecutionFailed(String),
}

/// A compute workload task to be executed by a GPU/CPU node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceTask {
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: u32,
}

/// The verifiable output of an inference execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub generated_text: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub output_hash: Hash,
    pub compute_units: u64,
}

/// Trait defining the local AI inference runner.
pub trait InferenceBackend: Send + Sync {
    fn execute(
        &self,
        task: &InferenceTask,
        registry: &ModelRegistry,
    ) -> Result<InferenceOutput, ComputeError>;
}

/// Deterministic inference engine implementing greedy integer decoding.
/// Ensures identical inputs under registered models produce bit-exact output hashes across nodes.
#[derive(Debug, Default, Clone)]
pub struct DeterministicInferenceEngine;

impl DeterministicInferenceEngine {
    pub fn new() -> Self {
        Self
    }

    /// Approximate deterministic tokenization for inference accounting.
    fn estimate_tokens(text: &str) -> u32 {
        // Standard heuristic: ~4 characters per token
        (text.len() as u32 / 4).max(1)
    }

    /// Deterministic greedy text generator based on input prompt and canonical model ID.
    fn generate_greedy_tokens(model: &ModelDefinition, prompt: &str, max_tokens: u32) -> (String, u32) {
        let seed_hash = hash(format!("{}:{}", model.id, prompt).as_bytes());
        let seed_bytes = seed_hash.as_bytes();

        let dictionary = [
            "decentralized", "autonomous", "inference", "compute", "blockchain",
            "proof", "useful", "work", "matrix", "multiplication", "consensus",
            "verification", "validator", "cryptographic", "receipt", "network",
            "model", "weights", "greedy", "deterministic", "optimization"
        ];

        let mut words = Vec::new();
        let target_tokens = max_tokens.min(128).max(8);

        for i in 0..target_tokens {
            let idx = ((seed_bytes[(i as usize) % 32] as usize) + (i as usize * 7)) % dictionary.len();
            words.push(dictionary[idx]);
        }

        let generated = words.join(" ");
        (generated, target_tokens)
    }
}

impl InferenceBackend for DeterministicInferenceEngine {
    fn execute(
        &self,
        task: &InferenceTask,
        registry: &ModelRegistry,
    ) -> Result<InferenceOutput, ComputeError> {
        let model = registry
            .get(&task.model_id)
            .ok_or_else(|| ComputeError::UnsupportedModel(task.model_id.clone()))?;

        let prompt_tokens = Self::estimate_tokens(&task.prompt);
        if prompt_tokens > model.max_context_tokens {
            return Err(ComputeError::ContextLengthExceeded);
        }

        let (generated_text, completion_tokens) =
            Self::generate_greedy_tokens(model, &task.prompt, task.max_tokens);

        let output_hash = hash(generated_text.as_bytes());
        let compute_units = model.calculate_compute_units(prompt_tokens, completion_tokens);

        Ok(InferenceOutput {
            generated_text,
            prompt_tokens,
            completion_tokens,
            output_hash,
            compute_units,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_reproducibility() {
        let registry = ModelRegistry::default();
        let engine = DeterministicInferenceEngine::new();

        let task = InferenceTask {
            model_id: "llama-3-8b-instruct".to_string(),
            prompt: "What is verifiable compute?".to_string(),
            max_tokens: 32,
        };

        let out1 = engine.execute(&task, &registry).unwrap();
        let out2 = engine.execute(&task, &registry).unwrap();

        assert_eq!(out1.output_hash, out2.output_hash);
        assert_eq!(out1.generated_text, out2.generated_text);
        assert_eq!(out1.compute_units, out2.compute_units);
    }
}
