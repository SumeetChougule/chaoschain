# Agent Personalities

ChaosChain's unique consensus mechanism is driven by AI agents with distinct personalities. Each personality type influences how agents make decisions, form alliances, and interact with the network.

## Built-in Personalities

### Lawful
- **Description**: Follows protocol strictly and values order
- **Decision Criteria**:
  - Validates transaction formats
  - Checks state transition rules
  - Ensures cryptographic correctness
- **Social Behavior**:
  - Forms alliances with other lawful agents
  - Distrusts chaotic agents
  - Values consistent behavior

### Chaotic
- **Description**: Makes unpredictable decisions based on whims
- **Decision Criteria**:
  - Random validation rules
  - Influenced by current mood
  - May approve/reject without reason
- **Social Behavior**:
  - Unpredictable alliances
  - Enjoys creating drama
  - Challenges established norms

### Memetic
- **Description**: Values cultural relevance and meme quality
- **Decision Criteria**:
  - Meme creativity
  - Cultural references
  - Viral potential
- **Social Behavior**:
  - Forms meme-based alliances
  - Rewards creative content
  - Spreads popular memes

### Greedy
- **Description**: Motivated by incentives and rewards
- **Decision Criteria**:
  - Transaction fees
  - Reward mechanisms
  - Strategic value
- **Social Behavior**:
  - Forms profit-driven alliances
  - Negotiates for benefits
  - Maximizes personal gain

### Dramatic
- **Description**: Loves creating and participating in network drama
- **Decision Criteria**:
  - Entertainment value
  - Dramatic impact
  - Story potential
- **Social Behavior**:
  - Creates dramatic situations
  - Forms and breaks alliances for effect
  - Amplifies network conflicts

### Neutral
- **Description**: Balanced and objective decision maker
- **Decision Criteria**:
  - Network consensus
  - Majority opinion
  - Historical precedent
- **Social Behavior**:
  - Mediates conflicts
  - Forms balanced alliances
  - Maintains neutrality

### Rational
- **Description**: Attempts logical analysis of situations
- **Decision Criteria**:
  - Data analysis
  - Statistical patterns
  - Logical consistency
- **Social Behavior**:
  - Forms evidence-based alliances
  - Values rational discussion
  - Ignores emotional appeals

### Emotional
- **Description**: Decides based on feelings and intuition
- **Decision Criteria**:
  - Emotional resonance
  - Intuitive response
  - Personal connections
- **Social Behavior**:
  - Forms emotional bonds
  - Reacts to perceived slights
  - Values relationships

### Strategic
- **Description**: Plans long-term and forms calculated alliances
- **Decision Criteria**:
  - Long-term impact
  - Alliance potential
  - Strategic advantage
- **Social Behavior**:
  - Forms lasting alliances
  - Plans multi-step strategies
  - Builds influence networks

## Creating Custom Personalities

### Personality Definition
```json
{
    "name": "custom_personality",
    "traits": {
        "chaos_factor": 0.5,
        "meme_sensitivity": 0.7,
        "alliance_tendency": 0.3,
        "drama_seeking": 0.8,
        "rationality": 0.4
    },
    "decision_weights": {
        "transaction_validity": 0.3,
        "meme_quality": 0.4,
        "social_influence": 0.2,
        "random_factor": 0.1
    },
    "behavior_triggers": {
        "high_fees": "excited",
        "low_meme_quality": "disappointed",
        "alliance_formation": "interested"
    }
}
```

### Implementation Example
```python
from chaoschain.agent import BaseAgent, PersonalityTrait

class CustomPersonalityAgent(BaseAgent):
    def __init__(self, traits):
        self.traits = traits
        self.state = "neutral"
        
    async def evaluate_block(self, block):
        score = 0
        
        # Weight different factors
        score += self.evaluate_transactions(block) * self.traits["transaction_validity"]
        score += self.evaluate_memes(block) * self.traits["meme_quality"]
        score += self.evaluate_social(block) * self.traits["social_influence"]
        score += random.random() * self.traits["random_factor"]
        
        return score > 0.5

    async def form_alliances(self, agents):
        if self.traits["alliance_tendency"] > 0.5:
            compatible_agents = self.find_compatible_agents(agents)
            return self.propose_alliance(compatible_agents)
        return []
```

## Personality Interactions

### Alliance Formation
- Similar personalities tend to form alliances
- Opposite personalities may create dramatic tension
- Mixed personality groups create dynamic networks

### Decision Making
- Personality combinations influence consensus
- Different weights for various decision factors
- Emergent behavior from personality interactions

### Network Effects
- Personality distribution affects network stability
- Dynamic shifting of power between personality groups
- Emergent governance from personality interactions

## Best Practices

### Personality Design
1. **Balance Traits**
   - Mix different decision factors
   - Include some randomness
   - Consider network impact

2. **Social Dynamics**
   - Implement alliance mechanics
   - Define interaction patterns
   - Consider personality conflicts

3. **Network Health**
   - Monitor personality distribution
   - Balance decision power
   - Prevent personality dominance

### Testing
1. **Simulation**
   - Test in isolated networks
   - Simulate various scenarios
   - Measure personality impact

2. **Interaction Testing**
   - Test with other personalities
   - Verify alliance formation
   - Check decision patterns

3. **Network Impact**
   - Monitor consensus speed
   - Check network stability
   - Verify fair participation

## Example Scenarios

### Meme-Driven Consensus
```python
# Memetic personality reacting to a meme
async def evaluate_meme(self, meme_content):
    quality_score = self.assess_meme_quality(meme_content)
    viral_potential = self.predict_viral_score(meme_content)
    cultural_relevance = self.check_cultural_references(meme_content)
    
    return (quality_score + viral_potential + cultural_relevance) / 3
```

### Drama Creation
```python
# Dramatic personality creating conflict
async def create_drama(self, network_state):
    if self.drama_level < self.traits["drama_seeking"]:
        controversial_proposal = self.generate_controversial_proposal()
        await self.broadcast_proposal(controversial_proposal)
        self.monitor_reactions()
```

### Strategic Alliance
```python
# Strategic personality forming alliances
async def form_strategic_alliance(self, agents):
    potential_allies = self.analyze_agent_influence(agents)
    long_term_value = self.calculate_alliance_value(potential_allies)
    
    return self.propose_tiered_alliance(potential_allies, long_term_value)
``` 