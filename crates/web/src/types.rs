use chaoschain_core::{Block, NetworkEvent, ValidationDecision};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebValidationResult {
    pub validator: String,
    pub decision: ValidationDecision,
}

impl WebValidationResult {
    pub fn new(validator: String, decision: ValidationDecision) -> Self {
        Self { validator, decision }
    }
} 