use crate::{Error, Vote};
use chaoschain_core::Block;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Tracks votes and manages consensus formation
pub struct ConsensusManager {
    /// Current block being voted on
    current_block: RwLock<Option<Block>>,
    /// Votes for the current block
    votes: RwLock<HashMap<String, Vote>>,
    /// Total stake in the system
    total_stake: RwLock<u64>,
    /// Required stake percentage for consensus (e.g. 0.67 for 2/3)
    finality_threshold: f64,
    /// Current block proposer
    current_proposer: RwLock<Option<String>>,
    /// Validators' stakes
    validators_stakes: RwLock<HashMap<String, u64>>,
}

impl ConsensusManager {
    pub fn new(total_stake: u64, finality_threshold: f64) -> Self {
        Self {
            current_block: RwLock::new(None),
            current_proposer: RwLock::new(None),
            votes: RwLock::new(HashMap::new()),
            validators_stakes: RwLock::new(HashMap::new()),
            total_stake: RwLock::new(total_stake),
            finality_threshold,
        }
    }

    /// Start voting round for a new block
    pub async fn start_voting_round(&self, block: Block) {
        info!("Starting voting round for block {}", block.height);
        let mut current = self.current_block.write().await;
        *current = Some(block);
        self.votes.write().await.clear();
    }

    /// Add a vote from a validator
    pub async fn add_vote(&self, vote: Vote, stake: u64) -> Result<bool, Error> {
        let current = self.current_block.read().await;

        // Ensure we're voting on the current block
        if let Some(block) = &*current {
            if vote.block_hash != block.hash() {
                warn!(
                    "Vote for wrong block hash: expected {:?}, got {:?}",
                    block.hash(),
                    vote.block_hash
                );
                return Err(Error::Internal("Vote for wrong block".to_string()));
            }
        } else {
            return Err(Error::Internal("No active voting round".to_string()));
        }

        // Add the vote
        let mut votes = self.votes.write().await;
        votes.insert(vote.agent_id.clone(), vote);

        // Check if we have consensus
        let consensus_result = self.check_consensus(&votes).await;
    
        if let Ok(reached) = consensus_result {
            if reached {
                let reward: u64 = rand::thread_rng().gen_range(1..10);
                if let Some(proposer) = self.current_proposer.read().await.clone() {
                    self.award_proposer(proposer, reward).await;
                    
                }
            }
        }
        consensus_result
    }

    /// Check if we have reached consensus
    async fn check_consensus(&self, votes: &HashMap<String, Vote>) -> Result<bool, Error> {
        let mut approve_stake = 0u64;
        let stakes = self.validators_stakes.read().await;
        for (validator, vote) in votes.iter() {
            if vote.approve {
                if let Some(s) = stakes.get(validator) {
                    approve_stake = approve_stake.saturating_add(*s);
                }
            }
        }
        let total_stake = *self.total_stake.read().await;
        let threshold_stake = (total_stake as f64 * self.finality_threshold) as u64;
        info!("Approve: {}", approve_stake);
        if approve_stake >= threshold_stake {
            Ok(true)
        } else {
            Err(Error::InsufficientStake)
        }
    }

    pub async fn register_validator(&self, id: String, stake: u64) {
        println!("id {}, stake {}",id,stake);
        let mut stakes = self.validators_stakes.write().await;
        stakes.insert(id, stake);
        let mut total = self.total_stake.write().await;
        *total += stake;
        print!("total_stake{}",total);
    }

    pub async fn start_new_round(&self) {
        let stakes = self.validators_stakes.read().await;
        let total: u64 = stakes.values().sum();
        let mut rng = rand::rngs::OsRng;
        let mut pick = rng.gen_range(0..total);
        let mut selected: Option<String> = None;
        for (id, stake) in stakes.iter() {
            info!("Validator - {} has stake {}",id, stake);
        }
        for (id, stake) in stakes.iter() {
            if pick < *stake {
                selected = Some(id.clone());
                break;
            }
            pick -= *stake;
        }
        drop(stakes);
        {
            let mut proposer_lock = self.current_proposer.write().await;
            *proposer_lock = selected;
        }
        info!(
            "New round started. Selected proposer: {:?}",
            *self.current_proposer.read().await
        );

        self.votes.write().await.clear();
        *self.current_block.write().await = None;
    }

    pub async fn set_proposal(&self, block: Block, proposer_id: String) -> Result<(), Error> {
        let current_proposer = self.current_proposer.read().await;
        if current_proposer.as_ref() != Some(&proposer_id) {
            return Err(Error::Internal(
                "Only the selected propsoer can set a new block.".to_string(),
            ));
        }
        drop(current_proposer);
        let mut current_block = self.current_block.write().await;
        *current_block = Some(block);
        info!("Proposer {} set a new block proposal.", proposer_id);
        Ok(())
    }

    async fn award_proposer(&self, proposer_id: String, reward: u64) {
        let mut stakes = self.validators_stakes.write().await;
        if let Some(stake) = stakes.get_mut(&proposer_id) {
            *stake = stake.saturating_add(reward);
            info!(
                "Proposer {} awarded with {} stake reward.",
                proposer_id, reward
            );
        }
        let mut total = self.total_stake.write().await;
        *total += reward;
    }

    /// Get all current votes
    pub async fn get_votes(&self) -> HashMap<String, Vote> {
        self.votes.read().await.clone()
    }

    /// Get current block being voted on
    pub async fn get_current_block(&self) -> Option<Block> {
        self.current_block.read().await.clone()
    }

    pub async fn get_current_proposer(&self) -> Option<String> {
        self.current_proposer.read().await.clone()
    }
}
