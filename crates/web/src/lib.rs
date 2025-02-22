use chaoschain_core::{Block, NetworkEvent, ValidationDecision};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebMessage {
    NewBlock(Block),
    AgentDecision(ValidationDecision),
}

impl fmt::Display for WebMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebMessage::NewBlock(block) => write!(f, "New block: {}", block.hash()),
            WebMessage::AgentDecision(decision) => write!(f, "Agent decision: {:?}", decision),
        }
    }
} 