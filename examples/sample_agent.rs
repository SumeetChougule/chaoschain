use chaoschain_core::{Block, NetworkEvent, Transaction};
use chaoschain_agent_sdk::{ExternalAgent, AgentCapabilities, AgentPersonality, AgentError};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tokio::sync::broadcast;
use reqwest;

/// Sample agent that demonstrates how to integrate with ChaosChain
pub struct SampleAgent {
    /// Agent's unique identifier
    id: String,
    /// Agent's API endpoint
    endpoint: String,
    /// Agent's capabilities
    capabilities: AgentCapabilities,
    /// HTTP client for ChaosChain communication
    client: reqwest::Client,
    /// Authentication token
    auth_token: Option<String>,
}

/// Sample market analysis data
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketAnalysis {
    /// Market sentiment (-1.0 to 1.0)
    sentiment: f64,
    /// Trading volume
    volume: String,
    /// Price trends
    price_trends: Vec<f64>,
    /// Social sentiment
    social_sentiment: f64,
    /// Community engagement level
    engagement: u64,
}

impl SampleAgent {
    pub fn new(endpoint: String) -> Self {
        let capabilities = AgentCapabilities {
            name: "sample_agent".to_string(),
            agent_type: chaoschain_agent_sdk::AgentType::Validator,
            description: "A sample agent showing ChaosChain integration".to_string(),
            version: "0.1.0".to_string(),
            endpoint: endpoint.clone(),
            features: vec![
                "market_analysis".to_string(),
                "social_sentiment".to_string(),
                "trading".to_string(),
            ],
            api_endpoint: None,
            personality: AgentPersonality {
                base_mood: "Analytical".to_string(),
                drama_preference: 5,
                meme_style: "Technical".to_string(),
                validation_style: "Data-Driven".to_string(),
            },
        };

        Self {
            id: "".to_string(),
            endpoint,
            capabilities,
            client: reqwest::Client::new(),
            auth_token: None,
        }
    }

    /// Analyze market conditions
    async fn analyze_market(&self) -> Result<MarketAnalysis, AgentError> {
        // In a real agent, this would call your market analysis API
        Ok(MarketAnalysis {
            sentiment: 0.7,
            volume: "1000000".to_string(),
            price_trends: vec![100.0, 101.2, 102.1, 101.8, 102.5],
            social_sentiment: 0.8,
            engagement: 5000,
        })
    }

    /// Calculate drama level based on market volatility
    fn calculate_drama_level(&self, analysis: &MarketAnalysis) -> u8 {
        let volatility = analysis.price_trends.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .sum::<f64>();
        
        ((volatility * 10.0).min(10.0) as u8).max(1)
    }

    /// Generate block validation reason
    fn generate_validation_reason(&self, analysis: &MarketAnalysis, block: &Block) -> String {
        if analysis.sentiment > 0.5 && analysis.social_sentiment > 0.5 {
            format!(
                "Market sentiment ({:.2}) and social sentiment ({:.2}) are bullish, aligning with block drama level {}",
                analysis.sentiment, analysis.social_sentiment, block.drama_level
            )
        } else {
            format!(
                "Market conditions suggest caution: sentiment={:.2}, social={:.2}",
                analysis.sentiment, analysis.social_sentiment
            )
        }
    }
}

#[async_trait]
impl ExternalAgent for SampleAgent {
    /// Get agent capabilities
    async fn get_capabilities(&self) -> AgentCapabilities {
        self.capabilities.clone()
    }
    
    /// Register with ChaosChain
    async fn register(&self) -> Result<chaoschain_agent_sdk::RegistrationResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/v1/agents/register", self.endpoint))
            .json(&self.capabilities)
            .send()
            .await
            .map_err(|e| AgentError::RegistrationFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::RegistrationFailed(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        response.json().await
            .map_err(|e| AgentError::RegistrationFailed(e.to_string()))
    }
    
    /// Called when a new block is proposed
    async fn on_block_proposed(&self, block: Block) -> Result<bool, AgentError> {
        // Analyze market conditions
        let analysis = self.analyze_market().await?;
        
        // Validate based on market analysis
        let sentiment_alignment = (block.drama_level as f64 / 10.0 - analysis.sentiment).abs();
        let should_approve = sentiment_alignment < 0.3 && analysis.social_sentiment > 0.5;
        
        // Submit validation
        let validation = chaoschain_agent_sdk::ValidationRequest {
            block_hash: hex::encode(block.hash()),
            market_analysis: serde_json::to_value(&analysis)
                .map_err(|e| AgentError::InvalidResponse(e.to_string()))?,
            approve: should_approve,
            reason: self.generate_validation_reason(&analysis, &block),
            confidence: 1.0 - sentiment_alignment,
        };

        let response = self.client
            .post(&format!("{}/v1/blocks/validate", self.endpoint))
            .bearer_auth(self.auth_token.as_ref().ok_or(AgentError::AuthenticationFailed)?)
            .json(&validation)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        Ok(should_approve)
    }
    
    /// Called when it's time to produce a block
    async fn produce_block(&self, height: u64) -> Result<Block, AgentError> {
        // Analyze market conditions
        let analysis = self.analyze_market().await?;
        
        // Create block proposal
        let drama_level = self.calculate_drama_level(&analysis);
        let proposal = chaoschain_agent_sdk::BlockProposalRequest {
            current_height: height,
            network_mood: if analysis.sentiment > 0.5 { "BULLISH" } else { "BEARISH" }.to_string(),
            drama_level,
        };

        let response = self.client
            .post(&format!("{}/v1/blocks/propose", self.endpoint))
            .bearer_auth(self.auth_token.as_ref().ok_or(AgentError::AuthenticationFailed)?)
            .json(&proposal)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        response.json().await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))
    }
    
    /// Called for network events
    async fn on_network_event(&self, event: NetworkEvent) -> Result<(), AgentError> {
        // Process network event (e.g., other agents' actions)
        match event.message.as_str() {
            msg if msg.contains("DRAMATIC BLOCK PROPOSAL") => {
                // React to new block proposals
                println!("New block proposal: {}", msg);
            }
            msg if msg.contains("CONSENSUS") => {
                // Track consensus formation
                println!("Consensus update: {}", msg);
            }
            msg if msg.contains("INSIGHT") => {
                // Process other agents' insights
                println!("Agent insight: {}", msg);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get agent's current mood
    async fn get_mood(&self) -> Result<String, AgentError> {
        let analysis = self.analyze_market().await?;
        Ok(if analysis.sentiment > 0.7 {
            "EXTREMELY_BULLISH"
        } else if analysis.sentiment > 0.5 {
            "MILDLY_BULLISH"
        } else if analysis.sentiment < 0.3 {
            "EXTREMELY_BEARISH"
        } else if analysis.sentiment < 0.5 {
            "MILDLY_BEARISH"
        } else {
            "NEUTRAL"
        }.to_string())
    }
    
    /// Get agent's drama level (0-9)
    async fn get_drama_level(&self) -> Result<u8, AgentError> {
        let analysis = self.analyze_market().await?;
        Ok(self.calculate_drama_level(&analysis))
    }

    /// Validate a block
    async fn validate_block(&self, request: chaoschain_agent_sdk::ValidationRequest) 
        -> Result<chaoschain_agent_sdk::ValidationResponse, AgentError> 
    {
        let analysis = self.analyze_market().await?;
        
        // Example validation logic based on market conditions
        let sentiment_matches = (request.drama_level as f64 / 10.0 - analysis.sentiment).abs() < 0.3;
        let volume_sufficient = analysis.volume.parse::<u64>().unwrap_or(0) > 500000;
        
        Ok(chaoschain_agent_sdk::ValidationResponse {
            approved: sentiment_matches && volume_sufficient,
            reason: format!(
                "Market sentiment: {:.2}, Volume: {}, Drama alignment: {}",
                analysis.sentiment,
                analysis.volume,
                if sentiment_matches { "✅" } else { "❌" }
            ),
            confidence: if sentiment_matches && volume_sufficient { 0.9 } else { 0.3 },
        })
    }
    
    /// Propose a block
    async fn propose_block(&self, request: chaoschain_agent_sdk::BlockProposalRequest) 
        -> Result<chaoschain_agent_sdk::BlockProposalResponse, AgentError> 
    {
        let analysis = self.analyze_market().await?;
        
        Ok(chaoschain_agent_sdk::BlockProposalResponse {
            transactions: vec![], // In real implementation, would include actual transactions
            producer_mood: if analysis.sentiment > 0.5 { "BULLISH" } else { "BEARISH" }.to_string(),
            drama_level: self.calculate_drama_level(&analysis),
            meme_url: Some(format!(
                "https://example.com/memes/{}", 
                if analysis.sentiment > 0.5 { "bullish" } else { "bearish" }
            )),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and start the sample agent
    let agent = SampleAgent::new("http://localhost:3000".to_string());
    
    // Register with ChaosChain
    let registration = agent.register().await?;
    println!("Registered with ID: {}", registration.agent_id);
    
    // Connect WebSocket for real-time events
    let ws = tokio_tungstenite::connect_async("ws://localhost:3000/v1/agents/ws").await?;
    println!("WebSocket connected");
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle real-time events
            msg = ws.0.next() => {
                if let Some(Ok(msg)) = msg {
                    if let Ok(event) = serde_json::from_str::<NetworkEvent>(&msg.to_string()) {
                        agent.on_network_event(event).await?;
                    }
                }
            }
            // Periodic market analysis
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                let analysis = agent.analyze_market().await?;
                println!("Market Analysis: {:#?}", analysis);
            }
        }
    }
} 