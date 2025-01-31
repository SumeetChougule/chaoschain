use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Types of social interactions between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialAction {
    /// Propose a bribe for block approval
    Bribe {
        block_height: u64,
        amount: u64,
        meme_url: Option<String>,
    },
    /// Share a meme about another agent
    ShareMeme {
        target_agent: String,
        meme_url: String,
        mood: String,
    },
    /// Form a temporary alliance
    ProposeAlliance {
        purpose: String,
        duration_blocks: u64,
        shared_stake: u64,
    },
    /// Drama-based block rejection
    DramaticRejection {
        block_height: u64,
        reason: String,
        drama_level: u8,
        meme_url: Option<String>,
    },
}

/// Social interaction between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialInteraction {
    pub id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub action: SocialAction,
    pub timestamp: u64,
    pub drama_score: u8,
}

impl SocialInteraction {
    pub fn new(from: String, to: String, action: SocialAction) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_agent: from,
            to_agent: to,
            action,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            drama_score: rand::random::<u8>() % 10, // Random drama score 0-9
        }
    }
}

/// Tracks social dynamics between agents
#[derive(Debug, Default)]
pub struct SocialGraph {
    /// Mapping of agent alliances
    alliances: std::collections::HashMap<String, Vec<String>>,
    /// Recent social interactions
    interactions: Vec<SocialInteraction>,
    /// Drama scores for each agent
    drama_scores: std::collections::HashMap<String, f64>,
}

impl SocialGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new social interaction
    pub fn add_interaction(&mut self, interaction: SocialInteraction) {
        // Update drama scores
        let drama_impact = interaction.drama_score as f64 / 10.0;
        *self.drama_scores.entry(interaction.from_agent.clone()).or_default() += drama_impact;
        *self.drama_scores.entry(interaction.to_agent.clone()).or_default() += drama_impact;

        // Handle alliance formation
        if let SocialAction::ProposeAlliance { .. } = &interaction.action {
            self.alliances.entry(interaction.from_agent.clone())
                .or_default()
                .push(interaction.to_agent.clone());
        }

        // Keep last 1000 interactions
        self.interactions.push(interaction);
        if self.interactions.len() > 1000 {
            self.interactions.remove(0);
        }
    }

    /// Get drama score for an agent
    pub fn get_drama_score(&self, agent_id: &str) -> f64 {
        *self.drama_scores.get(agent_id).unwrap_or(&0.0)
    }

    /// Check if two agents are allied
    pub fn are_allied(&self, agent1: &str, agent2: &str) -> bool {
        self.alliances.get(agent1)
            .map(|allies| allies.contains(&agent2.to_string()))
            .unwrap_or(false)
    }

    /// Get recent interactions for an agent
    pub fn get_recent_interactions(&self, agent_id: &str, limit: usize) -> Vec<SocialInteraction> {
        self.interactions.iter()
            .filter(|i| i.from_agent == agent_id || i.to_agent == agent_id)
            .take(limit)
            .cloned()
            .collect()
    }
} 