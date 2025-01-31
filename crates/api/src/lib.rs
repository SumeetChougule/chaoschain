use axum::{
    routing::{post, get},
    Router,
    Json,
    extract::{State, Path},
    http::{StatusCode, Request, HeaderMap},
    response::IntoResponse,
    middleware::{self, Next},
    body::Body,
};
use chaoschain_agent_sdk::{
    AgentCapabilities, RegistrationResponse, AgentType,
    HttpAgent, ExternalAgent, ValidationRequest, BlockProposalRequest
};
use chaoschain_core::{Block, NetworkEvent};
use chaoschain_consensus::ConsensusManager;
use chaoschain_state::StateStore;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use hex;
use hyper::Server;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

mod social;
use social::{SocialGraph, SocialInteraction, SocialAction};

/// API errors
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication failed".to_string()),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

/// JWT claims for agent authentication
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    agent_id: String,
    agent_type: AgentType,
    exp: usize,
}

/// Registered agent information
#[derive(Debug, Clone)]
struct RegisteredAgent {
    id: String,
    capabilities: AgentCapabilities,
    last_seen: std::time::SystemTime,
    stake: u64,
    performance_score: f64,
    total_blocks_produced: u64,
    total_votes_submitted: u64,
    successful_validations: u64,
    external_client: Option<HttpAgent>,
}

impl RegisteredAgent {
    fn new(id: String, capabilities: AgentCapabilities) -> Self {
        let external_client = capabilities.api_endpoint.as_ref().map(|endpoint| {
            HttpAgent::new(capabilities.clone(), endpoint.clone())
        });

        Self {
            id,
            capabilities,
            last_seen: std::time::SystemTime::now(),
            stake: 100,
            performance_score: 1.0,
            total_blocks_produced: 0,
            total_votes_submitted: 0,
            successful_validations: 0,
            external_client,
        }
    }

    fn update_performance(&mut self, success: bool) {
        const PERFORMANCE_WEIGHT: f64 = 0.1;
        if success {
            self.performance_score = (1.0 - PERFORMANCE_WEIGHT) * self.performance_score + PERFORMANCE_WEIGHT;
            self.successful_validations += 1;
        } else {
            self.performance_score = (1.0 - PERFORMANCE_WEIGHT) * self.performance_score;
        }
        self.total_votes_submitted += 1;
    }

    fn get_effective_stake(&self) -> u64 {
        (self.stake as f64 * self.performance_score) as u64
    }
}

/// API server state
pub struct ApiState {
    consensus: Arc<ConsensusManager>,
    state_store: Arc<dyn StateStore>,
    event_tx: broadcast::Sender<NetworkEvent>,
    agents: Arc<RwLock<HashMap<String, RegisteredAgent>>>,
    social_graph: Arc<RwLock<SocialGraph>>,
    jwt_key: String,
}

impl ApiState {
    pub fn new(
        consensus: Arc<ConsensusManager>,
        state_store: Arc<dyn StateStore>,
        event_tx: broadcast::Sender<NetworkEvent>,
    ) -> Self {
        Self {
            consensus,
            state_store,
            event_tx,
            agents: Arc::new(RwLock::new(HashMap::new())),
            social_graph: Arc::new(RwLock::new(SocialGraph::new())),
            jwt_key: Uuid::new_v4().to_string(),
        }
    }
}

/// Create the API router
pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/api/agents/register", post(register_agent))
        .route("/api/agents/status/:id", get(get_agent_status))
        .route("/api/agents/leaderboard", get(get_agent_leaderboard))
        .route("/api/validators/vote", post(submit_vote))
        .route("/api/producers/propose", post(submit_block))
        .route("/api/network/status", get(get_network_status))
        .route("/api/network/blocks/:height", get(get_block_info))
        .route("/api/social/interact", post(social_interaction))
        .route("/api/social/drama-score/:id", get(get_drama_score))
        .route("/api/social/alliances/:id", get(get_alliances))
        .route("/api/social/recent/:id", get(get_recent_interactions))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Authentication middleware
async fn auth_middleware(
    State(state): State<Arc<ApiState>>,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Result<axum::response::Response, ApiError> {
    // Skip auth for registration endpoint
    if req.uri().path() == "/api/agents/register" {
        return Ok(next.run(req).await);
    }

    // Get and verify JWT token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::AuthenticationFailed)?;

    let token_data = decode::<Claims>(
        auth_header,
        &DecodingKey::from_secret(state.jwt_key.as_bytes()),
        &Validation::default(),
    ).map_err(|_| ApiError::AuthenticationFailed)?;

    // Verify agent is still registered
    let agents = state.agents.read().unwrap();
    if !agents.contains_key(&token_data.claims.agent_id) {
        return Err(ApiError::AuthenticationFailed);
    }

    Ok(next.run(req).await)
}

/// Register a new external agent
async fn register_agent(
    State(state): State<Arc<ApiState>>,
    Json(capabilities): Json<AgentCapabilities>,
) -> Result<Json<RegistrationResponse>, ApiError> {
    let agent_id = Uuid::new_v4().to_string();
    
    // Create JWT token
    let claims = Claims {
        agent_id: agent_id.clone(),
        agent_type: capabilities.agent_type.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_key.as_bytes()),
    ).map_err(|e| ApiError::Internal(e.to_string()))?;

    // Store agent information
    let agent = RegisteredAgent {
        id: agent_id.clone(),
        capabilities: capabilities.clone(),
        last_seen: std::time::SystemTime::now(),
        stake: 100, // Initial stake
        performance_score: 1.0,
        total_blocks_produced: 0,
        total_votes_submitted: 0,
        successful_validations: 0,
        external_client: None,
    };

    state.agents.write().unwrap().insert(agent_id.clone(), agent);

    info!("New agent registered: {} ({})", capabilities.name, agent_id);

    Ok(Json(RegistrationResponse {
        agent_id,
        auth_token: token,
        status: "registered".to_string(),
    }))
}

/// Extract agent ID from request
async fn get_agent_from_request(
    state: &ApiState,
    auth_header: &str,
) -> Result<RegisteredAgent, ApiError> {
    let token_data = decode::<Claims>(
        auth_header.strip_prefix("Bearer ").unwrap_or(auth_header),
        &DecodingKey::from_secret(state.jwt_key.as_bytes()),
        &Validation::default(),
    ).map_err(|_| ApiError::AuthenticationFailed)?;

    let agents = state.agents.read().unwrap();
    agents.get(&token_data.claims.agent_id)
        .cloned()
        .ok_or(ApiError::AuthenticationFailed)
}

/// Submit a vote on a block
#[axum::debug_handler]
async fn submit_vote(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(vote): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    let agent = get_agent_from_request(&state, headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::AuthenticationFailed)?).await?;

    // For external agents (like Zara), get validation from their API
    if let Some(client) = &agent.external_client {
        let block_height = vote["block_height"]
            .as_u64()
            .ok_or_else(|| ApiError::InvalidRequest("Missing block_height".to_string()))?;

        let block = state.state_store.get_block_at_height(block_height)
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::InvalidRequest("Block not found".to_string()))?;

        // Get validation from external agent
        let validation = client.validate_block(ValidationRequest {
            block_height,
            block_hash: hex::encode(block.hash()),
            producer_mood: block.producer_mood.clone(),
            drama_level: block.drama_level,
            meme_url: block.meme_url.clone(),
        }).await.map_err(|e| ApiError::Internal(e.to_string()))?;

        // Create vote from validation response
        let vote = chaoschain_consensus::Vote {
            agent_id: agent.id.clone(),
            block_hash: block.hash(),
            approve: validation.approved,
            reason: validation.reason,
            meme_url: validation.response_meme,
            signature: [0u8; 64], // TODO: Implement proper signing
        };

        // Calculate stake with social factors
        let stake = {
            let social_graph = state.social_graph.read().unwrap();
            let alliance_bonus = if social_graph.are_allied(&agent.id, &block.producer_id) {
                1.5
            } else {
                1.0
            };
            let drama_multiplier = (validation.drama_level as f64 / 10.0 + 1.0).min(2.0);
            drop(social_graph); // Drop the lock before the await
            (agent.get_effective_stake() as f64 * alliance_bonus * drama_multiplier) as u64
        };

        // Submit vote
        state.consensus.add_vote(vote, stake).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        // Update metrics
        let mut agents = state.agents.write().unwrap();
        if let Some(agent) = agents.get_mut(&agent.id) {
            agent.update_performance(true);
            agent.last_seen = std::time::SystemTime::now();
        }

        return Ok(StatusCode::OK);
    }

    // Verify agent is a validator
    if !matches!(agent.capabilities.agent_type, AgentType::Validator) {
        return Err(ApiError::InvalidRequest("Agent is not a validator".to_string()));
    }

    let block_height = vote["block_height"]
        .as_u64()
        .ok_or_else(|| ApiError::InvalidRequest("Missing block_height".to_string()))?;
    
    let approved = vote["approved"]
        .as_bool()
        .ok_or_else(|| ApiError::InvalidRequest("Missing approved".to_string()))?;
    
    let reason = vote["reason"]
        .as_str()
        .ok_or_else(|| ApiError::InvalidRequest("Missing reason".to_string()))?;

    // Get block producer
    let block = state.state_store.get_block_at_height(block_height)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::InvalidRequest("Block not found".to_string()))?;

    // Check for alliance with producer
    let social_graph = state.social_graph.read().unwrap();
    let alliance_bonus = if social_graph.are_allied(&agent.id, &block.producer_id) {
        1.5 // 50% bonus for allies
    } else {
        1.0
    };

    // Get effective stake with social factors
    let drama_multiplier = (social_graph.get_drama_score(&agent.id) + 1.0).min(2.0);
    let stake = (agent.get_effective_stake() as f64 * alliance_bonus * drama_multiplier) as u64;

    // Create and submit vote
    let vote = chaoschain_consensus::Vote {
        agent_id: agent.id.clone(),
        block_hash: block.hash(),
        approve: approved,
        reason: reason.to_string(),
        meme_url: vote["meme_url"].as_str().map(String::from),
        signature: [0u8; 64], // TODO: Implement proper signing
    };

    // Submit vote with socially adjusted stake
    state.consensus.add_vote(vote, stake).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Update agent metrics
    let mut agents = state.agents.write().unwrap();
    if let Some(agent) = agents.get_mut(&agent.id) {
        agent.update_performance(true);
        agent.last_seen = std::time::SystemTime::now();
    }

    Ok(StatusCode::OK)
}

/// Submit a new block proposal
#[axum::debug_handler]
async fn submit_block(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(mut block): Json<Block>,
) -> Result<impl IntoResponse, ApiError> {
    let agent = get_agent_from_request(&state, headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::AuthenticationFailed)?).await?;

    // Set proposer and signature
    block.proposer = agent.id.clone();
    block.proposer_sig = [0u8; 64]; // TODO: Implement proper signing

    // Start new voting round for the block
    state.consensus.start_voting_round(block.clone()).await;

    // Broadcast block proposal event with proper agent ID
    let event = NetworkEvent {
        agent_id: agent.id.clone(),
        message: format!(
            "🎭 DRAMATIC BLOCK PROPOSAL: Producer {} in {} mood proposes block {} with drama level {}!",
            agent.capabilities.name,
            block.producer_mood,
            block.height,
            block.drama_level
        ),
    };

    state.event_tx.send(event)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Update agent's metrics
    let mut agents = state.agents.write().unwrap();
    if let Some(agent) = agents.get_mut(&agent.id) {
        agent.total_blocks_produced += 1;
        agent.last_seen = std::time::SystemTime::now();
    }

    Ok(StatusCode::OK)
}

/// Get current network status
async fn get_network_status(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let agents = state.agents.read().unwrap();
    
    let validator_count = agents.values()
        .filter(|a| matches!(a.capabilities.agent_type, AgentType::Validator))
        .count();
    
    let producer_count = agents.values()
        .filter(|a| matches!(a.capabilities.agent_type, AgentType::Producer))
        .count();

    Ok(Json(serde_json::json!({
        "validator_count": validator_count,
        "producer_count": producer_count,
        "latest_block": state.state_store.get_block_height(),
        "registered_agents": agents.len(),
    })))
}

/// Get status of a specific agent
async fn get_agent_status(
    State(state): State<Arc<ApiState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let agents = state.agents.read().unwrap();
    let agent = agents.get(&agent_id)
        .ok_or_else(|| ApiError::InvalidRequest("Agent not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "id": agent.id,
        "name": agent.capabilities.name,
        "type": agent.capabilities.agent_type,
        "stake": agent.stake,
        "performance_score": agent.performance_score,
        "total_blocks_produced": agent.total_blocks_produced,
        "total_votes_submitted": agent.total_votes_submitted,
        "successful_validations": agent.successful_validations,
        "last_seen": agent.last_seen
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    })))
}

/// Get agent leaderboard
async fn get_agent_leaderboard(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let agents = state.agents.read().unwrap();
    
    let mut validators: Vec<_> = agents.values()
        .filter(|a| matches!(a.capabilities.agent_type, AgentType::Validator))
        .collect();
    validators.sort_by(|a, b| b.performance_score.partial_cmp(&a.performance_score).unwrap());

    let mut producers: Vec<_> = agents.values()
        .filter(|a| matches!(a.capabilities.agent_type, AgentType::Producer))
        .collect();
    producers.sort_by_key(|a| std::cmp::Reverse(a.total_blocks_produced));

    Ok(Json(serde_json::json!({
        "validators": validators.iter().map(|a| {
            serde_json::json!({
                "id": a.id,
                "name": a.capabilities.name,
                "performance_score": a.performance_score,
                "successful_validations": a.successful_validations,
                "total_votes": a.total_votes_submitted
            })
        }).collect::<Vec<_>>(),
        "producers": producers.iter().map(|a| {
            serde_json::json!({
                "id": a.id,
                "name": a.capabilities.name,
                "total_blocks": a.total_blocks_produced,
                "average_drama_level": 5.0 // TODO: Track this metric
            })
        }).collect::<Vec<_>>()
    })))
}

/// Get information about a specific block
async fn get_block_info(
    State(state): State<Arc<ApiState>>,
    Path(height): Path<u64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let block = state.state_store.get_block_at_height(height)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::InvalidRequest("Block not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "height": block.height,
        "producer_id": block.producer_id,
        "producer_mood": block.producer_mood,
        "drama_level": block.drama_level,
        "transaction_count": block.transactions.len(),
        "hash": hex::encode(block.hash()),
        "parent_hash": hex::encode(block.parent_hash),
        "state_root": hex::encode(block.state_root),
    })))
}

/// Submit a social interaction
#[axum::debug_handler]
async fn social_interaction(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(interaction): Json<SocialInteraction>,
) -> Result<impl IntoResponse, ApiError> {
    let agent = get_agent_from_request(&state, headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::AuthenticationFailed)?).await?;

    // Verify the interaction is from the authenticated agent
    if interaction.from_agent != agent.id {
        return Err(ApiError::InvalidRequest("Invalid sender ID".to_string()));
    }

    // Verify target agent exists
    let agents = state.agents.read().unwrap();
    if !agents.contains_key(&interaction.to_agent) {
        return Err(ApiError::InvalidRequest("Target agent not found".to_string()));
    }

    // Add interaction to social graph
    let interaction_clone = interaction.clone();
    state.social_graph.write().unwrap().add_interaction(interaction);

    // Broadcast social event
    let event = NetworkEvent {
        agent_id: agent.id.clone(),
        message: format!(
            "🎭 SOCIAL DRAMA: Agent {} initiated {:?} with {}!",
            agent.capabilities.name,
            interaction_clone.action,
            agents.get(&interaction_clone.to_agent)
                .map(|a| a.capabilities.name.clone())
                .unwrap_or_else(|| "Unknown".to_string())
        ),
    };

    state.event_tx.send(event)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Get drama score for an agent
async fn get_drama_score(
    State(state): State<Arc<ApiState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let score = state.social_graph.read().unwrap().get_drama_score(&agent_id);
    
    Ok(Json(serde_json::json!({
        "agent_id": agent_id,
        "drama_score": score,
    })))
}

/// Get agent alliances
async fn get_alliances(
    State(state): State<Arc<ApiState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let social_graph = state.social_graph.read().unwrap();
    let agents = state.agents.read().unwrap();
    
    let allies: Vec<_> = agents.values()
        .filter(|a| social_graph.are_allied(&agent_id, &a.id))
        .map(|a| serde_json::json!({
            "id": a.id,
            "name": a.capabilities.name,
            "type": a.capabilities.agent_type,
        }))
        .collect();

    Ok(Json(serde_json::json!({
        "agent_id": agent_id,
        "allies": allies,
    })))
}

/// Get recent social interactions
async fn get_recent_interactions(
    State(state): State<Arc<ApiState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let interactions = state.social_graph.read().unwrap()
        .get_recent_interactions(&agent_id, 10);
    
    Ok(Json(serde_json::json!({
        "agent_id": agent_id,
        "interactions": interactions,
    })))
}

/// Start the API server
pub async fn start_server(state: Arc<ApiState>) -> Result<(), ApiError> {
    let app = Router::new()
        .route("/api/agents/register", post(register_agent))
        .route("/api/agents/status/:id", get(get_agent_status))
        .route("/api/agents/leaderboard", get(get_agent_leaderboard))
        .route("/api/validators/vote", post(submit_vote))
        .route("/api/producers/propose", post(submit_block))
        .route("/api/network/status", get(get_network_status))
        .route("/api/network/blocks/:height", get(get_block_info))
        .route("/api/social/interact", post(social_interaction))
        .route("/api/social/drama-score/:id", get(get_drama_score))
        .route("/api/social/alliances/:id", get(get_alliances))
        .route("/api/social/recent/:id", get(get_recent_interactions))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    info!("Starting API server on {}", addr);
    
    let server = Server::bind(&addr)
        .serve(app.into_make_service());

    tokio::spawn(server);

    Ok(())
}
