use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::{Vote, Error};
use chaoschain_core::Block;
use tracing::{info, warn};

pub struct ConsensusState {
    pub threshold: u64,
    pub total_stake: u64,
    pub votes: HashMap<[u8; 32], (Vote, u64)>, // block_hash -> (vote, stake)
    pub current_block: Option<Block>,
}

/// Tracks votes and manages consensus formation
pub struct ConsensusManager {
    state: RwLock<ConsensusState>,
}

impl ConsensusManager {
    pub fn new(total_stake: u64, finality_threshold: f64) -> Self {
        let threshold = (total_stake as f64 * finality_threshold) as u64;
        Self {
            state: RwLock::new(ConsensusState {
                threshold,
                total_stake,
                votes: HashMap::new(),
                current_block: None,
            }),
        }
    }

    /// Update the consensus threshold
    pub async fn update_consensus_threshold(&self, threshold: u64) {
        let mut state = self.state.write().await;
        state.threshold = threshold;
    }

    /// Start voting round for a new block
    pub async fn start_voting_round(&self, block: Block) {
        info!("Starting voting round for block {}", block.height);
        let mut state = self.state.write().await;
        state.current_block = Some(block);
        state.votes.clear();
    }

    /// Add a vote from a validator
    pub async fn add_vote(&self, vote: Vote, stake: u64) -> Result<bool, Error> {
        let mut state = self.state.write().await;
        
        // Check if we have enough stake
        if stake == 0 {
            return Err(Error::InsufficientStake);
        }

        // Add vote
        state.votes.insert(vote.block_hash, (vote, stake));

        // Calculate total approving stake
        let approving_stake: u64 = state.votes.values()
            .filter(|(vote, _)| vote.approve)
            .map(|(_, stake)| stake)
            .sum();

        // Check if we've reached consensus
        Ok(approving_stake >= state.threshold)
    }

    /// Get all current votes
    pub async fn get_votes(&self) -> HashMap<[u8; 32], (Vote, u64)> {
        let state = self.state.read().await;
        state.votes.clone()
    }

    /// Get current block being voted on
    pub async fn get_current_block(&self) -> Option<Block> {
        let state = self.state.read().await;
        state.current_block.clone()
    }
} 