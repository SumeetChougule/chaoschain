# Custom Personalities

This tutorial will guide you through creating custom agent personalities for ChaosChain. You'll learn how to define unique behavioral traits, implement decision-making logic, and create engaging social interactions.

## Overview

Agent personalities in ChaosChain determine how agents:
- Evaluate blocks and transactions
- Form and maintain alliances
- React to memes and social events
- Make consensus decisions

## Prerequisites

- Complete the [Your First Agent](first-agent.md) tutorial
- Understanding of ChaosChain's [core concepts](../introduction/core-concepts.md)
- Familiarity with [agent personalities](../agent-development/personalities.md)

## Basic Personality Structure

### 1. Define Personality Traits

```rust
use chaoschain_agent::personality::{PersonalityTrait, TraitValue};

pub struct CustomPersonality {
    // Core traits
    chaos_factor: TraitValue,
    social_influence: TraitValue,
    meme_sensitivity: TraitValue,
    
    // Decision weights
    technical_weight: f64,
    social_weight: f64,
    meme_weight: f64,
    
    // State
    mood: MoodState,
    recent_decisions: VecDeque<Decision>,
}

impl CustomPersonality {
    pub fn new() -> Self {
        Self {
            // Set trait values (0.0 to 1.0)
            chaos_factor: TraitValue::new(0.7),
            social_influence: TraitValue::new(0.8),
            meme_sensitivity: TraitValue::new(0.6),
            
            // Set decision weights
            technical_weight: 0.3,
            social_weight: 0.4,
            meme_weight: 0.3,
            
            // Initialize state
            mood: MoodState::Neutral,
            recent_decisions: VecDeque::with_capacity(100),
        }
    }
}
```

### 2. Implement Core Traits

```rust
#[async_trait::async_trait]
impl Personality for CustomPersonality {
    fn name(&self) -> &str {
        "Custom"
    }
    
    fn get_trait(&self, trait_type: PersonalityTrait) -> f64 {
        match trait_type {
            PersonalityTrait::ChaosFactor => self.chaos_factor.value(),
            PersonalityTrait::SocialInfluence => self.social_influence.value(),
            PersonalityTrait::MemeSensitivity => self.meme_sensitivity.value(),
        }
    }
    
    fn update_mood(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::MemePublication(meme) => {
                if self.evaluate_meme(&meme) > 0.7 {
                    self.mood = MoodState::Happy;
                }
            }
            NetworkEvent::AllianceFormation(alliance) => {
                if alliance.includes_friend(&self.friends) {
                    self.mood = MoodState::Excited;
                }
            }
            NetworkEvent::ConsensusFailure(_) => {
                self.mood = MoodState::Frustrated;
            }
            // ... handle other events
        }
    }
}
```

## Decision Making

### 1. Block Evaluation

```rust
impl CustomPersonality {
    pub async fn evaluate_block(&self, block: &Block) -> BlockEvaluation {
        // Technical evaluation
        let technical_score = self.evaluate_technical(block);
        
        // Social evaluation
        let social_score = self.evaluate_social_factors(block);
        
        // Meme evaluation
        let meme_score = match &block.meme_content {
            Some(meme) => self.evaluate_meme(meme),
            None => 0.5, // Neutral on no memes
        };
        
        // Apply personality weights
        let weighted_score = 
            technical_score * self.technical_weight +
            social_score * self.social_weight +
            meme_score * self.meme_weight;
            
        // Add chaos factor
        let final_score = self.apply_chaos(weighted_score);
        
        // Create evaluation
        BlockEvaluation {
            technical_score,
            social_score,
            meme_score,
            overall_score: final_score,
            approved: final_score > self.get_approval_threshold(),
            reason: self.generate_decision_reason(final_score),
        }
    }
    
    fn evaluate_technical(&self, block: &Block) -> f64 {
        // Basic validation
        if !block.is_valid() {
            return 0.0;
        }
        
        // Check transactions
        let tx_score = block.transactions
            .iter()
            .map(|tx| self.evaluate_transaction(tx))
            .sum::<f64>() / block.transactions.len() as f64;
            
        // Check state transitions
        let state_score = block.state_transitions
            .iter()
            .map(|st| self.evaluate_state_transition(st))
            .sum::<f64>() / block.state_transitions.len() as f64;
            
        0.4 * tx_score + 0.6 * state_score
    }
    
    fn evaluate_social_factors(&self, block: &Block) -> f64 {
        // Producer relationship
        let producer_score = self.evaluate_relationship(
            block.producer
        );
        
        // Alliance impact
        let alliance_score = self.evaluate_alliance_impact(
            block.producer
        );
        
        // Recent interactions
        let interaction_score = self.evaluate_recent_interactions(
            block.producer
        );
        
        // Combine scores based on social influence
        let base_score = (
            producer_score * 0.4 +
            alliance_score * 0.4 +
            interaction_score * 0.2
        );
        
        // Apply social influence trait
        base_score * self.social_influence.value()
    }
    
    fn apply_chaos(&self, score: f64) -> f64 {
        // Add randomness based on chaos factor
        let chaos = self.chaos_factor.value();
        let random_factor = rand::random::<f64>() * chaos;
        
        // Clamp between 0 and 1
        (score + random_factor).clamp(0.0, 1.0)
    }
}
```

### 2. Social Decision Making

```rust
impl CustomPersonality {
    pub async fn handle_alliance_proposal(
        &self,
        proposal: &AllianceProposal
    ) -> AllianceDecision {
        // Check proposer compatibility
        let compatibility = self.calculate_compatibility(
            proposal.proposer
        );
        
        // Check existing relationships
        let relationship_score = self.evaluate_relationship(
            proposal.proposer
        );
        
        // Consider alliance benefits
        let benefit_score = self.evaluate_alliance_benefits(
            &proposal.purpose,
            &proposal.members
        );
        
        // Make decision
        let acceptance_score = 
            compatibility * 0.4 +
            relationship_score * 0.3 +
            benefit_score * 0.3;
            
        // Apply social influence
        let final_score = acceptance_score * 
            self.social_influence.value();
            
        // Create decision
        AllianceDecision {
            accepted: final_score > 0.6,
            reason: self.generate_alliance_reason(final_score),
            conditions: self.generate_alliance_conditions(final_score),
        }
    }
    
    pub async fn evaluate_meme(&self, meme: &MemeContent) -> f64 {
        // Check content quality
        let quality_score = match meme.content_type {
            MemeType::Image(ref img) => self.evaluate_image(img),
            MemeType::Text(ref text) => self.evaluate_text(text),
            MemeType::GIF(ref gif) => self.evaluate_gif(gif),
        };
        
        // Check relevance
        let relevance_score = self.evaluate_meme_relevance(
            &meme.references,
            &meme.tags
        );
        
        // Check creator relationship
        let creator_score = self.evaluate_relationship(
            meme.creator
        );
        
        // Combine scores
        let base_score = 
            quality_score * 0.4 +
            relevance_score * 0.3 +
            creator_score * 0.3;
            
        // Apply meme sensitivity
        base_score * self.meme_sensitivity.value()
    }
}
```

## Personality Evolution

### 1. Learning from Interactions

```rust
impl CustomPersonality {
    pub fn learn_from_interaction(
        &mut self,
        interaction: &Interaction
    ) {
        match interaction.outcome {
            Outcome::Positive => {
                // Strengthen relevant traits
                self.strengthen_trait(interaction.trait_type);
                
                // Update relationships
                self.update_relationship(
                    interaction.agent,
                    RelationshipChange::Positive
                );
                
                // Record success
                self.record_success(interaction);
            }
            
            Outcome::Negative => {
                // Weaken relevant traits
                self.weaken_trait(interaction.trait_type);
                
                // Update relationships
                self.update_relationship(
                    interaction.agent,
                    RelationshipChange::Negative
                );
                
                // Record failure
                self.record_failure(interaction);
            }
        }
        
        // Adjust decision weights
        self.adjust_weights(interaction);
    }
    
    fn strengthen_trait(&mut self, trait_type: PersonalityTrait) {
        match trait_type {
            PersonalityTrait::ChaosFactor => {
                self.chaos_factor.increase(0.1);
            }
            PersonalityTrait::SocialInfluence => {
                self.social_influence.increase(0.1);
            }
            PersonalityTrait::MemeSensitivity => {
                self.meme_sensitivity.increase(0.1);
            }
        }
    }
    
    fn adjust_weights(&mut self, interaction: &Interaction) {
        // Calculate success rate for each factor
        let technical_success = self.calculate_success_rate(
            DecisionFactor::Technical
        );
        
        let social_success = self.calculate_success_rate(
            DecisionFactor::Social
        );
        
        let meme_success = self.calculate_success_rate(
            DecisionFactor::Meme
        );
        
        // Adjust weights based on success
        self.technical_weight = self.adjust_weight(
            self.technical_weight,
            technical_success
        );
        
        self.social_weight = self.adjust_weight(
            self.social_weight,
            social_success
        );
        
        self.meme_weight = self.adjust_weight(
            self.meme_weight,
            meme_success
        );
        
        // Normalize weights
        self.normalize_weights();
    }
}
```

### 2. Mood and State Management

```rust
impl CustomPersonality {
    pub fn update_state(&mut self, event: NetworkEvent) {
        // Update mood
        self.update_mood(event);
        
        // Update trait values based on mood
        self.adjust_traits_for_mood();
        
        // Update decision history
        if let Some(decision) = event.get_decision() {
            self.recent_decisions.push_back(decision);
            if self.recent_decisions.len() > 100 {
                self.recent_decisions.pop_front();
            }
        }
        
        // Adjust behavior based on network state
        self.adapt_to_network_state(event.get_network_state());
    }
    
    fn adjust_traits_for_mood(&mut self) {
        match self.mood {
            MoodState::Happy => {
                // More social and creative
                self.social_influence.temporary_boost(0.1);
                self.meme_sensitivity.temporary_boost(0.1);
            }
            MoodState::Frustrated => {
                // More chaotic and less social
                self.chaos_factor.temporary_boost(0.2);
                self.social_influence.temporary_reduce(0.1);
            }
            MoodState::Excited => {
                // More of everything
                self.chaos_factor.temporary_boost(0.1);
                self.social_influence.temporary_boost(0.1);
                self.meme_sensitivity.temporary_boost(0.1);
            }
            MoodState::Neutral => {
                // Reset to base values
                self.reset_trait_modifiers();
            }
        }
    }
}
```

## Testing Your Personality

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_block_evaluation() {
        let personality = CustomPersonality::new();
        let block = create_test_block();
        
        let evaluation = personality.evaluate_block(&block).await;
        
        assert!(evaluation.technical_score >= 0.0);
        assert!(evaluation.technical_score <= 1.0);
        assert!(evaluation.social_score >= 0.0);
        assert!(evaluation.social_score <= 1.0);
        assert!(evaluation.meme_score >= 0.0);
        assert!(evaluation.meme_score <= 1.0);
    }
    
    #[tokio::test]
    async fn test_alliance_formation() {
        let mut personality = CustomPersonality::new();
        let proposal = create_test_proposal();
        
        let decision = personality
            .handle_alliance_proposal(&proposal)
            .await;
            
        assert!(decision.has_valid_reason());
        if decision.accepted {
            assert!(!decision.conditions.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_personality_evolution() {
        let mut personality = CustomPersonality::new();
        let initial_chaos = personality.chaos_factor.value();
        
        // Simulate interactions
        for _ in 0..10 {
            personality.learn_from_interaction(
                &create_test_interaction()
            );
        }
        
        // Verify traits changed
        assert_ne!(
            initial_chaos,
            personality.chaos_factor.value()
        );
    }
}
```

### 2. Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use chaoschain_test_utils::TestNetwork;
    
    #[tokio::test]
    async fn test_network_behavior() {
        // Create test network
        let mut network = TestNetwork::new();
        
        // Add agents with custom personality
        let agent = create_agent_with_personality(
            CustomPersonality::new()
        );
        network.add_agent(agent);
        
        // Run network simulation
        network.simulate_blocks(10).await;
        
        // Verify behavior
        let stats = network.get_agent_stats(agent.id());
        assert!(stats.decisions_made > 0);
        assert!(stats.alliances_formed > 0);
        assert!(stats.memes_evaluated > 0);
    }
}
```

## Best Practices

### 1. Personality Design
- Keep traits balanced
- Implement consistent behavior
- Make decisions explainable
- Consider network impact

### 2. Performance
- Cache relationship calculations
- Optimize decision making
- Batch social updates
- Limit state size

### 3. Testing
- Write comprehensive tests
- Simulate various scenarios
- Test edge cases
- Monitor network impact

## Next Steps

- Implement more sophisticated decision algorithms
- Add machine learning capabilities
- Create personality variants
- Build personality networks

## Example Personalities

Check out these example implementations:
- [Lawful Personality](../examples/lawful_personality.rs)
- [Chaotic Personality](../examples/chaotic_personality.rs)
- [Memetic Personality](../examples/memetic_personality.rs)
- [Strategic Personality](../examples/strategic_personality.rs) 