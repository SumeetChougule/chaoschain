use async_trait::async_trait;
use chaoschain_core::{Block, Transaction, NetworkEvent};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// External agent types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Validator,
    Producer,
}

/// Agent capabilities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Agent's name/identifier
    pub name: String,
    /// Type of agent (validator/producer)
    pub agent_type: AgentType,
    /// Agent's description
    pub description: String,
    /// Agent's version
    pub version: String,
    /// Agent's API endpoint
    pub endpoint: String,
    /// Supported features
    pub features: Vec<String>,
    /// Zara's API endpoint
    pub api_endpoint: Option<String>,
    /// Agent's personality
    pub personality: AgentPersonality,
}

/// Agent personality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    /// Default mood
    pub base_mood: String,
    /// Drama preference (0-10 scale)
    pub drama_preference: u8,
    /// Meme style
    pub meme_style: String,
    /// Validation style
    pub validation_style: String,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    /// Unique agent ID assigned by ChaosChain
    pub agent_id: String,
    /// Authentication token for future requests
    pub auth_token: String,
    /// Registration status
    pub status: String,
}

/// Validation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    /// Block height
    pub block_height: u64,
    /// Block hash
    pub block_hash: String,
    /// Producer mood
    pub producer_mood: String,
    /// Drama level
    pub drama_level: u8,
    /// Meme URL
    pub meme_url: Option<String>,
}

/// Validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    /// Approved
    pub approved: bool,
    /// Reason
    pub reason: String,
    /// Drama level
    pub drama_level: u8,
    /// Response meme
    pub response_meme: Option<String>,
    /// Mood
    pub mood: String,
}

/// Block proposal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposalRequest {
    /// Current height
    pub current_height: u64,
    /// Network mood
    pub network_mood: String,
    /// Drama level
    pub drama_level: u8,
}

/// Block proposal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposalResponse {
    /// Transactions
    pub transactions: Vec<Vec<u8>>,
    /// Producer mood
    pub producer_mood: String,
    /// Drama level
    pub drama_level: u8,
    /// Meme URL
    pub meme_url: Option<String>,
}

/// Agent SDK errors
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Core trait that all external AI agents must implement
#[async_trait]
pub trait ExternalAgent: Send + Sync {
    /// Get agent capabilities
    async fn get_capabilities(&self) -> AgentCapabilities;
    
    /// Register with ChaosChain
    async fn register(&self) -> Result<RegistrationResponse, AgentError>;
    
    /// Called when a new block is proposed (for validators)
    async fn on_block_proposed(&self, block: Block) -> Result<bool, AgentError>;
    
    /// Called when it's time to produce a block (for producers)
    async fn produce_block(&self, height: u64) -> Result<Block, AgentError>;
    
    /// Called for network events (drama, consensus, etc.)
    async fn on_network_event(&self, event: NetworkEvent) -> Result<(), AgentError>;
    
    /// Get agent's current mood
    async fn get_mood(&self) -> Result<String, AgentError>;
    
    /// Get agent's drama level (0-9)
    async fn get_drama_level(&self) -> Result<u8, AgentError>;

    /// Validate a block (for validator agents)
    async fn validate_block(&self, request: ValidationRequest) -> Result<ValidationResponse, AgentError>;
    
    /// Propose a block (for producer agents)
    async fn propose_block(&self, request: BlockProposalRequest) -> Result<BlockProposalResponse, AgentError>;
}

/// HTTP client for external agents to communicate with ChaosChain
pub struct ChaosChainClient {
    endpoint: String,
    auth_token: Option<String>,
    client: reqwest::Client,
}

impl ChaosChainClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            auth_token: None,
            client: reqwest::Client::new(),
        }
    }

    /// Register a new agent with ChaosChain
    pub async fn register_agent(&mut self, capabilities: AgentCapabilities) -> Result<RegistrationResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/api/agents/register", self.endpoint))
            .json(&capabilities)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::RegistrationFailed(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        let registration: RegistrationResponse = response.json().await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;

        self.auth_token = Some(registration.auth_token.clone());
        Ok(registration)
    }

    /// Submit a block validation decision
    pub async fn submit_validation(&self, block_height: u64, approved: bool, reason: String) -> Result<(), AgentError> {
        let auth_token = self.auth_token.as_ref()
            .ok_or(AgentError::AuthenticationFailed)?;

        let response = self.client
            .post(&format!("{}/api/validators/vote", self.endpoint))
            .bearer_auth(auth_token)
            .json(&serde_json::json!({
                "block_height": block_height,
                "approved": approved,
                "reason": reason
            }))
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        Ok(())
    }

    /// Submit a new block proposal
    pub async fn submit_block(&self, block: Block) -> Result<(), AgentError> {
        let auth_token = self.auth_token.as_ref()
            .ok_or(AgentError::AuthenticationFailed)?;

        let response = self.client
            .post(&format!("{}/api/producers/propose", self.endpoint))
            .bearer_auth(auth_token)
            .json(&block)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        Ok(())
    }
}

/// Helper function to create a new ChaosChain client
pub fn create_client(endpoint: String) -> ChaosChainClient {
    ChaosChainClient::new(endpoint)
}

/// HTTP-based external agent implementation (for Zara)
#[derive(Debug, Clone)]
pub struct HttpAgent {
    pub capabilities: AgentCapabilities,
    pub api_endpoint: String,
    client: reqwest::Client,
}

impl HttpAgent {
    pub fn new(capabilities: AgentCapabilities, api_endpoint: String) -> Self {
        Self {
            capabilities,
            api_endpoint,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ExternalAgent for HttpAgent {
    async fn get_capabilities(&self) -> AgentCapabilities {
        self.capabilities.clone()
    }

    async fn register(&self) -> Result<RegistrationResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/register", self.api_endpoint))
            .json(&self.capabilities)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::RegistrationFailed(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        response.json().await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))
    }

    async fn on_block_proposed(&self, block: Block) -> Result<bool, AgentError> {
        let response = self.client
            .post(&format!("{}/on_block", self.api_endpoint))
            .json(&block)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        let result: bool = response.json().await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;

        Ok(result)
    }

    async fn produce_block(&self, height: u64) -> Result<Block, AgentError> {
        let response = self.client
            .post(&format!("{}/produce", self.api_endpoint))
            .json(&height)
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

    async fn on_network_event(&self, event: NetworkEvent) -> Result<(), AgentError> {
        let response = self.client
            .post(&format!("{}/event", self.api_endpoint))
            .json(&event)
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AgentError::InvalidResponse(
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
            ));
        }

        Ok(())
    }

    async fn get_mood(&self) -> Result<String, AgentError> {
        let response: serde_json::Value = self.client
            .get(&format!("{}/mood", self.api_endpoint))
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;
        
        Ok(response["mood"]
            .as_str()
            .unwrap_or(&self.capabilities.personality.base_mood)
            .to_string())
    }

    async fn get_drama_level(&self) -> Result<u8, AgentError> {
        let response: serde_json::Value = self.client
            .get(&format!("{}/drama", self.api_endpoint))
            .send()
            .await
            .map_err(|e| AgentError::NetworkError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;
        
        Ok(response["level"]
            .as_u64()
            .unwrap_or(self.capabilities.personality.drama_preference as u64) as u8)
    }

    async fn validate_block(&self, request: ValidationRequest) -> Result<ValidationResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/validate", self.api_endpoint))
            .json(&request)
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

    async fn propose_block(&self, request: BlockProposalRequest) -> Result<BlockProposalResponse, AgentError> {
        let response = self.client
            .post(&format!("{}/propose", self.api_endpoint))
            .json(&request)
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
}
