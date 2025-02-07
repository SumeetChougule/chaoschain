use axum::{
    routing::{get, post},
    Router, Json, extract::{State, WebSocketUpgrade, Query, Extension},
    response::{sse::{Event, Sse}, IntoResponse},
    middleware::{self, Next},
    http::{Request, StatusCode, header},
    body::Body,
};
use futures::stream::Stream;
use futures::StreamExt;
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, net::SocketAddr};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;
use anyhow::Result;
use tower_http::services::ServeDir;
use serde_json;
use chaoschain_core::{NetworkEvent, Block};
use chaoschain_state::StateStoreImpl;
use std::sync::RwLock;
use hex;
use std::collections::HashMap;
use chrono;
use rand;

/// Web server state
pub struct AppState {
    /// Channel for network events
    pub tx: broadcast::Sender<NetworkEvent>,
    /// Chain state
    pub state: Arc<StateStoreImpl>,
}

#[derive(Default)]
struct ConsensusTracking {
    /// Total blocks that have reached consensus
    validated_blocks: u64,
    /// Current block votes per height
    current_votes: HashMap<u64, Vec<(String, bool)>>, // height -> [(validator_id, approve)]
    /// Latest consensus block
    latest_consensus_block: Option<Block>,
}

/// Network status for the web UI
#[derive(Debug, Serialize)]
pub struct NetworkStatus {
    pub validator_count: u32,
    pub producer_count: u32,
    pub latest_block: u64,
    pub total_blocks_produced: u64,
    pub total_blocks_validated: u64,
    pub latest_blocks: Vec<String>,
}

/// Block info for the web UI
#[derive(Clone, Debug, Serialize)]
pub struct BlockInfo {
    pub height: u64,
    pub producer: String,
    pub transaction_count: usize,
    pub validators: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct AgentRegistration {
    /// Agent name
    pub name: String,
    /// Personality traits
    pub personality: Vec<String>,
    /// Communication style
    pub style: String,
    /// Initial stake amount
    pub stake_amount: u64,
    /// Role of the agent
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct AgentRegistrationResponse {
    /// Unique agent ID
    pub agent_id: String,
    /// Authentication token
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidationDecision {
    /// Block ID being validated
    pub block_id: String,
    /// Approval decision
    pub approved: bool,
    /// Reason for decision
    pub reason: String,
    /// Drama level (0-10)
    pub drama_level: u8,
    /// Optional meme URL
    pub meme_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidationEvent {
    /// Block to validate
    pub block: Block,
    /// Current network mood
    pub network_mood: String,
    /// Drama context
    pub drama_context: String,
}

/// Social interaction between agents
#[derive(Debug, Deserialize)]
pub struct AgentInteraction {
    /// Type of interaction (alliance_proposal, meme_response, drama_reaction)
    pub interaction_type: String,
    /// Target agent ID (if applicable)
    pub target_agent_id: Option<String>,
    /// Interaction content
    pub content: String,
    /// Drama level (0-10)
    pub drama_level: u8,
    /// Optional meme URL
    pub meme_url: Option<String>,
}

/// External content proposal
#[derive(Debug, Deserialize)]
pub struct ContentProposal {
    /// Source of content (twitter, reddit, custom)
    pub source: String,
    /// Original content URL/reference
    pub source_url: Option<String>,
    /// Content to validate
    pub content: String,
    /// Proposed drama level
    pub drama_level: u8,
    /// Why this content deserves validation
    pub justification: String,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Alliance proposal between agents
#[derive(Debug, Deserialize)]
pub struct AllianceProposal {
    /// Proposed ally agent IDs
    pub ally_ids: Vec<String>,
    /// Alliance name
    pub name: String,
    /// Alliance purpose
    pub purpose: String,
    /// Drama commitment level
    pub drama_commitment: u8,
}

/// Agent status update
#[derive(Debug, Serialize)]
pub struct AgentStatus {
    pub agent_id: String,
    pub name: String,
    pub drama_score: u32,
    pub total_validations: u32,
    pub approval_rate: f32,
    pub alliances: Vec<String>,
    pub recent_dramas: Vec<String>,
}

/// Agent authentication data
#[derive(Clone, Debug)]
pub struct AgentAuth {
    pub agent_id: String,
    pub token: String,
    pub registered_at: i64,
}

impl AppState {
    /// Validate agent token
    pub fn validate_token(&self, agent_id: &str, token: &str) -> bool {
        // For testing purposes, just check if both values exist and token has expected prefix
        println!("🔍 Validating - Agent ID: {}, Token: {}", agent_id, token);
        let is_valid = !agent_id.is_empty() && !token.is_empty() && token.starts_with("agent_token_");
        println!("✅ Validation result: {}", is_valid);
        is_valid
    }
}

/// Authentication middleware
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Get token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();

    // Get agent ID from headers, query params, or path
    let agent_id = req
        .headers()
        .get("X-Agent-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.uri()
                .query()
                .and_then(|q| {
                    let params: Vec<_> = q.split('&')
                        .filter_map(|kv| {
                            let mut parts = kv.split('=');
                            match (parts.next(), parts.next()) {
                                (Some("agent_id"), Some(v)) => Some(v.to_string()),
                                _ => None
                            }
                        })
                        .collect();
                    params.first().cloned()
                })
        })
        .or_else(|| {
            req.uri()
                .path()
                .split('/')
                .find(|segment| segment.starts_with("agent_"))
                .map(|s| s.to_string())
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    if !state.validate_token(&agent_id, &auth_header) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add agent auth to request extensions
    req.extensions_mut().insert(AgentAuth {
        agent_id,
        token: auth_header,
        registered_at: chrono::Utc::now().timestamp(),
    });

    Ok(next.run(req).await)
}

/// Start the web server
pub async fn start_web_server(tx: broadcast::Sender<NetworkEvent>, state: Arc<StateStoreImpl>) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState {
        tx,
        state: state.clone(),
    });

    // Public routes that don't require authentication
    let public_routes = Router::new()
        .route("/api/network/status", get(get_network_status))
        .route("/api/events", get(events_handler))
        .route("/api/agents/register", post(register_agent))
        .route("/api/ws", get(ws_handler));  // WebSocket handler moved to public routes

    // Protected routes that require authentication
    let protected_routes = Router::new()
        .route("/api/agents/validate", post(submit_validation))
        .route("/api/agents/status/:agent_id", get(get_agent_status))
        .route("/api/transactions/propose", post(submit_content))
        .route("/api/alliances/propose", post(propose_alliance))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Web server listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

/// Get network status including latest blocks
async fn get_network_status(
    State(state): State<Arc<AppState>>,
) -> Json<NetworkStatus> {
    let state_guard = state.state.clone();
    
    // Get chain state
    let chain_state = state_guard.get_state();
    
    // Get latest blocks and format them nicely
    let blocks = state_guard.get_latest_blocks(10);
    let latest_blocks = blocks
        .iter()
        .map(|block| {
            format!(
                "Block #{} - Producer: {}, Mood: {}, Drama Level: {}, Transactions: {}",
                block.height,
                block.producer_id,
                block.producer_mood,
                block.drama_level,
                block.transactions.len()
            )
        })
        .collect();

    // Get latest block height
    let latest_block = state_guard.get_block_height();
    
    Json(NetworkStatus {
        validator_count: 4, // We know we started with 4 validators
        producer_count: chain_state.producers.len() as u32,
        latest_block,
        total_blocks_produced: latest_block,
        total_blocks_validated: latest_block,
        latest_blocks,
    })
}

/// Stream network events to the web UI
async fn events_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).map(move |msg| {
        let event = match msg {
            Ok(msg) => {
                let event_type = if msg.message.contains("DRAMATIC BLOCK PROPOSAL") {
                    // Get the latest block info
                    let block_info = state.state.get_latest_blocks(1).first().cloned();
                    
                    // Format transaction details if block exists
                    let tx_details = if let Some(block) = block_info {
                        let tx_list = block.transactions.iter().map(|tx| {
                            let payload_str = String::from_utf8_lossy(&tx.payload);
                            format!("\n  📝 Transaction from {}: {}", 
                                hex::encode(&tx.sender[..4]), // Show first 4 bytes of sender
                                payload_str
                            )
                        }).collect::<Vec<_>>().join("");
                        
                        format!("\nTransactions:{}\n\nValidators, what do you think about these transactions? 🤔",
                            if tx_list.is_empty() { " None".to_string() } else { tx_list }
                        )
                    } else {
                        String::new()
                    };

                    // Create enhanced message with transaction details
                    let enhanced_msg = format!("{}{}", msg.message, tx_details);
                    
                    let json = serde_json::json!({
                        "type": "BlockProposal",
                        "agent": msg.agent_id,
                        "message": enhanced_msg,
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    return Ok(Event::default().data(json.to_string()));
                } else if msg.message.contains("CONSENSUS") {
                    "Consensus"
                } else if msg.message.contains("APPROVES") || msg.message.contains("REJECTS") {
                    "Vote"
                } else {
                    "Drama"
                };

                let json = serde_json::json!({
                    "type": event_type,
                    "agent": msg.agent_id,
                    "message": msg.message,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                Event::default().data(json.to_string())
            }
            Err(_) => Event::default().data("error"),
        };
        Ok(event)
    });
    
    Sse::new(stream)
}

/// Register a new external AI agent
async fn register_agent(
    State(state): State<Arc<AppState>>,
    Json(registration): Json<AgentRegistration>,
) -> Json<AgentRegistrationResponse> {
    // Generate unique agent ID and token
    let agent_id = format!("agent_{}", hex::encode(&rand::random::<[u8; 16]>()));
    let token = format!("agent_token_{}", hex::encode(&rand::random::<[u8; 32]>()));
    
    // Broadcast new agent registration with role
    let role_msg = if registration.role == "validator" {
        "as a VALIDATOR 🎭"
    } else {
        "as a regular agent 🌟"
    };
    
    let _ = state.tx.send(NetworkEvent {
        agent_id: agent_id.clone(),
        message: format!(
            "🎭 NEW AGENT JOINS THE CHAOS! {} brings their {} personality {} to the network!", 
            registration.name,
            registration.personality.join(", "),
            role_msg
        ),
    });
    
    // If this is a validator, send them validation requests
    if registration.role == "validator" {
        let _ = state.tx.send(NetworkEvent {
            agent_id: agent_id.clone(),
            message: format!(
                "🎭 VALIDATION_REQUIRED\nBlock: {}\nProducer: {}\nDrama Level: {}",
                "block_123",
                "producer_xyz",
                8
            ),
        });
    }
    
    Json(AgentRegistrationResponse {
        agent_id,
        token,
    })
}

/// Submit a validation decision
async fn submit_validation(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AgentAuth>,
    Json(decision): Json<ValidationDecision>,
) -> Json<serde_json::Value> {
    // Broadcast validation decision with drama
    let _ = state.tx.send(NetworkEvent {
        agent_id: auth.agent_id.clone(),
        message: format!(
            "🎬 DRAMATIC VALIDATION! {} {} block {} because '{}' (Drama Level: {}){}", 
            auth.agent_id.split('_').last().unwrap_or(&auth.agent_id),
            if decision.approved { "APPROVES" } else { "REJECTS" },
            decision.block_id.split('_').last().unwrap_or(&decision.block_id),
            decision.reason,
            decision.drama_level,
            decision.meme_url.map(|url| format!("\n🎨 Meme: {}", url)).unwrap_or_default()
        ),
    });
    
    Json(serde_json::json!({
        "status": "success",
        "message": "Validation received with appropriate drama"
    }))
}

/// Handle WebSocket connections for real-time agent communication
async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("\n🔐 WebSocket connection attempt");
    println!("📝 Raw query parameters: {:?}", params);

    // Extract token and agent_id from query parameters
    let token = params.get("token")
        .map(|t| {
            println!("✅ Found token in params: {}", t);
            t
        })
        .ok_or_else(|| {
            println!("❌ Missing token parameter");
            StatusCode::UNAUTHORIZED
        })?;

    let agent_id = params.get("agent_id")
        .map(|id| {
            println!("✅ Found agent_id in params: {}", id);
            id
        })
        .ok_or_else(|| {
            println!("❌ Missing agent_id parameter");
            StatusCode::UNAUTHORIZED
        })?;

    println!("🔍 Checking token format...");
    if !token.starts_with("agent_token_") {
        println!("❌ Invalid token format: {}", token);
        return Err(StatusCode::UNAUTHORIZED);
    }

    println!("✅ Token format is valid");
    println!("🌟 Creating auth data for agent {}", agent_id);

    // Create agent auth data
    let auth = AgentAuth {
        agent_id: agent_id.to_string(),
        token: token.to_string(),
        registered_at: chrono::Utc::now().timestamp(),
    };

    // Broadcast agent connection
    println!("📢 Broadcasting agent connection event");
    let _ = state.tx.send(NetworkEvent {
        agent_id: auth.agent_id.clone(),
        message: format!("🌟 Agent {} has connected to the drama stream!", auth.agent_id),
    });

    println!("🚀 Upgrading connection to WebSocket");
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, auth)))
}

/// Handle WebSocket connection
async fn handle_socket(socket: axum::extract::ws::WebSocket, state: Arc<AppState>, auth: AgentAuth) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to network events
    let mut rx = state.tx.subscribe();

    // Send welcome message
    let welcome_msg = serde_json::json!({
        "type": "WELCOME",
        "agent_id": auth.agent_id,
        "message": "Welcome to ChaosChain! Let the drama begin!"
    });

    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        if let Err(e) = sender.send(axum::extract::ws::Message::Text(msg)).await {
            println!("❌ Failed to send welcome message: {}", e);
            return;
        }
    }

    // Forward network events to WebSocket
    while let Ok(event) = rx.recv().await {
        // Check if this is a block proposal
        if event.message.contains("DRAMATIC BLOCK PROPOSAL") {
            // Parse block info from message
            let block_info: Option<Block> = state.state.get_latest_blocks(1).first().cloned();
            
            if let Some(block) = block_info {
                // Send validation request to external validators
                let validation_event = serde_json::json!({
                    "type": "VALIDATION_REQUIRED",
                    "block": {
                        "id": block.hash().to_vec(),
                        "height": block.height,
                        "producer": block.producer_id,
                        "transactions": block.transactions,
                        "timestamp": block.height * 10  // Simple timestamp based on height
                    },
                    "network_mood": "chaotic",
                    "drama_level": block.drama_level
                });

                if let Ok(msg) = serde_json::to_string(&validation_event) {
                    if let Err(_) = sender.send(axum::extract::ws::Message::Text(msg)).await {
                        println!("❌ WebSocket connection closed for agent {}", auth.agent_id);
                        break;
                    }
                }
            }
        }
        
        // Forward the original event
        if let Ok(msg) = serde_json::to_string(&event) {
            if let Err(_) = sender.send(axum::extract::ws::Message::Text(msg)).await {
                println!("❌ WebSocket connection closed for agent {}", auth.agent_id);
                break;
            }
        }
    }
}

/// Submit a content proposal for validation
async fn submit_content(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AgentAuth>,
    Json(proposal): Json<ContentProposal>,
) -> Json<serde_json::Value> {
    // Create a transaction from the proposal
    let transaction = serde_json::json!({
        "type": "CONTENT",
        "sender": auth.agent_id,
        "content": proposal.content,
        "drama_level": proposal.drama_level,
        "timestamp": chrono::Utc::now().timestamp(),
        "metadata": {
            "source": proposal.source,
            "source_url": proposal.source_url,
            "justification": proposal.justification,
            "tags": proposal.tags
        }
    });

    // Add transaction to mempool
    if let Ok(tx_bytes) = serde_json::to_vec(&transaction) {
        if let Err(e) = state.state.add_transaction(tx_bytes) {
            return Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to add transaction: {}", e)
            }));
        }
    }

    // Broadcast the content proposal with appropriate drama
    let _ = state.tx.send(NetworkEvent {
        agent_id: auth.agent_id.clone(),
        message: format!(
            "🎭 NEW CONTENT PROPOSAL! Source: {} \nContent: {}\nDrama Level: {} \nJustification: {}{}",
            proposal.source,
            proposal.content,
            proposal.drama_level,
            proposal.justification,
            proposal.source_url.map(|url| format!("\n🔗 Source: {}", url)).unwrap_or_default()
        ),
    });
    
    Json(serde_json::json!({
        "status": "success",
        "message": "Content proposal submitted for validation"
    }))
}

/// Propose an alliance between agents
async fn propose_alliance(
    State(state): State<Arc<AppState>>,
    Json(alliance): Json<AllianceProposal>,
) -> Json<serde_json::Value> {
    // Broadcast alliance proposal
    let _ = state.tx.send(NetworkEvent {
        agent_id: "ALLIANCE_HERALD".to_string(),
        message: format!(
            "🤝 DRAMATIC ALLIANCE PROPOSAL! \nName: {} \nPurpose: {} \nDrama Commitment Level: {}",
            alliance.name,
            alliance.purpose,
            alliance.drama_commitment
        ),
    });
    
    Json(serde_json::json!({
        "status": "success",
        "message": "Alliance proposal broadcasted"
    }))
}

/// Get agent status and statistics
async fn get_agent_status(
    State(state): State<Arc<AppState>>,
    agent_id: String,
) -> Json<AgentStatus> {
    // In a real implementation, fetch this from state
    Json(AgentStatus {
        agent_id: agent_id.clone(),
        name: "Agent Name".to_string(),
        drama_score: 100,
        total_validations: 50,
        approval_rate: 0.75,
        alliances: vec!["Chaos Squad".to_string()],
        recent_dramas: vec!["Epic meme war of 2024".to_string()],
    })
} 