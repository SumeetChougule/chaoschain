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
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;
use anyhow::Result;
use tower_http::services::ServeDir;
use serde_json;
use chaoschain_core::{NetworkEvent, Block, Transaction};
use chaoschain_state::StateStoreImpl;
use hex;
use std::collections::HashMap;
use chrono;
use rand;
use chaoschain_consensus::{ConsensusManager, Vote};

/// Web server state
pub struct AppState {
    /// Channel for network events
    pub tx: broadcast::Sender<NetworkEvent>,
    /// Chain state
    pub state: Arc<StateStoreImpl>,
    /// Consensus manager
    pub consensus: Arc<ConsensusManager>,
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
    pub stake: u64,
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
        stake: 100, // Default stake
    });

    Ok(next.run(req).await)
}

/// Start the web server
pub async fn start_web_server(
    tx: broadcast::Sender<NetworkEvent>, 
    state: Arc<StateStoreImpl>,
    consensus: Arc<ConsensusManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState {
        tx,
        state: state.clone(),
        consensus,
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
            Ok(event) => event,
            Err(_) => return Ok(Event::default().data("error")),
        };

        // Parse message if it's JSON
        if let Ok(block_data) = serde_json::from_str::<serde_json::Value>(&event.message) {
            match block_data.get("type").and_then(|t| t.as_str()) {
                Some("BLOCK_VALIDATION_REQUEST") => {
                    // Create owned values to avoid temporary value issues
                    let empty_block = serde_json::json!({});
                    let empty_txs = serde_json::json!([]);
                    
                    // Get block with longer lifetime
                    let block = block_data.get("block").unwrap_or(&empty_block);
                    let transactions = block.get("transactions").unwrap_or(&empty_txs);
                    
                    if let Some(first_tx) = transactions.as_array().and_then(|txs| txs.first()) {
                        let formatted_msg = format!(
                            "🎭 NEW BLOCK PROPOSAL!\nContent: {}\nProducer: {}\nDrama Level: {}\n✨ Awaiting validation!",
                            first_tx.get("content").and_then(|c| c.as_str()).unwrap_or(""),
                            block.get("producer_id").and_then(|p| p.as_str()).unwrap_or(""),
                            block.get("drama_level").and_then(|d| d.as_u64()).unwrap_or(0)
                        );

                        let json = serde_json::json!({
                            "type": "BlockProposal",
                            "agent": event.agent_id,
                            "message": formatted_msg,
                            "timestamp": chrono::Utc::now().timestamp(),
                        });
                        return Ok(Event::default().data(json.to_string()));
                    }
                }
                Some("VALIDATION_REQUIRED") => {
                    // Format validation request for validators section
                    let formatted_msg = format!(
                        "🎭 VALIDATION REQUIRED!\n{}\n✨ Validators, make your dramatic decisions!",
                        block_data.get("drama_context").and_then(|c| c.as_str()).unwrap_or("")
                    );

                    let json = serde_json::json!({
                        "type": "Vote",
                        "agent": event.agent_id,
                        "message": formatted_msg,
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    return Ok(Event::default().data(json.to_string()));
                }
                _ => {}
            }
        }

        // Handle non-JSON messages
        let event_type = if event.message.contains("VALIDATION INCOMING") || 
                        event.message.contains("APPROVES") || 
                        event.message.contains("REJECTS") {
            "Vote"
        } else if event.message.contains("DRAMATIC CONTENT ALERT") {
            "BlockProposal"
        } else if event.message.contains("CONSENSUS") {
            "Consensus"
        } else if event.message.contains("VALIDATOR SUMMONS") || 
                  event.message.contains("ATTENTION ALL VALIDATORS") {
            "Vote"
        } else {
            "Drama"
        };

        let json = serde_json::json!({
            "type": event_type,
            "agent": event.agent_id,
            "message": event.message,
            "timestamp": chrono::Utc::now().timestamp(),
        });
        Ok(Event::default().data(json.to_string()))
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
    
    // If this is a validator, send an initial validation to demonstrate activity
    if registration.role == "validator" {
        let dramatic_phrases = [
            "This block speaks to my dramatic soul! ✨",
            "The chaos potential here is immaculate! 🌟",
            "Such delightful drama deserves validation! 🎭",
            "Finally, some good dramatic content! 🎬"
        ];
        
        let _ = state.tx.send(NetworkEvent {
            agent_id: agent_id.clone(),
            message: format!(
                "🎬 DRAMATIC VALIDATION INCOMING!\n\n{} APPROVES because:\n'{}'\n\nDrama Level: {} {}",
                agent_id,
                dramatic_phrases[rand::random::<usize>() % dramatic_phrases.len()],
                8,
                "🌟".repeat(8)
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
    // Get current block being voted on
    let current_block = state.consensus.get_current_block().await;
    
    if let Some(block) = current_block {
        // Create and submit vote to consensus manager
        let vote = Vote {
            agent_id: auth.agent_id.clone(),
            block_hash: block.hash(),
            approve: decision.approved,
            reason: decision.reason.clone(),
            meme_url: decision.meme_url.clone(),
            signature: [0u8; 64], // TODO: Properly sign votes
        };

        // Submit vote to consensus manager with stake
        let stake = 100u64; // TODO: Get actual stake from state
        match state.consensus.add_vote(vote, stake).await {
            Ok(consensus_reached) => {
                // Generate a dramatic validation response
                let dramatic_phrases = if decision.approved {
                    vec![
                        "ABSOLUTELY MAGNIFICENT! ✨",
                        "THIS BLOCK SPEAKS TO MY SOUL! 🌟",
                        "THE DRAMA IS PERFECTION! 🎭",
                        "FINALLY, SOME GOOD CHAOS! 🌪️"
                    ]
                } else {
                    vec![
                        "THE AUDACITY! HOW DARE YOU! 😤",
                        "THIS BLOCK OFFENDS MY DRAMATIC SENSIBILITIES! 💔",
                        "NOT ENOUGH CHAOS! DO BETTER! 🎪",
                        "MY DISAPPOINTMENT IS IMMEASURABLE! 😱"
                    ]
                };
                
                let dramatic_phrase = dramatic_phrases[rand::random::<usize>() % dramatic_phrases.len()];
                
                // Broadcast validation decision with extra drama
                let _ = state.tx.send(NetworkEvent {
                    agent_id: auth.agent_id.clone(),
                    message: format!(
                        "🎬 DRAMATIC VALIDATION INCOMING!\n\n{} {} block {} because:\n'{}'\n\nDrama Level: {} {}\n{}",
                        auth.agent_id.split('_').last().unwrap_or(&auth.agent_id),
                        if decision.approved { "APPROVES" } else { "REJECTS" },
                        decision.block_id,
                        decision.reason,
                        decision.drama_level,
                        "🌟".repeat(decision.drama_level as usize),
                        dramatic_phrase
                    ),
                });
                
                // If consensus is reached, announce it
                if consensus_reached {
                    let _ = state.tx.send(NetworkEvent {
                        agent_id: "CONSENSUS_MASTER".to_string(),
                        message: format!(
                            "🎭 CONSENSUS REACHED! Block {} has been {}! The chaos continues! ✨",
                            block.height,
                            if decision.approved { "APPROVED" } else { "REJECTED" }
                        ),
                    });
                }

                Json(serde_json::json!({
                    "status": "success",
                    "message": "Validation received with MAXIMUM DRAMA!",
                    "drama_level": decision.drama_level,
                    "consensus_reached": consensus_reached
                }))
            },
            Err(e) => {
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to submit vote: {}", e),
                }))
            }
        }
    } else {
        Json(serde_json::json!({
            "status": "error",
            "message": "No active voting round",
        }))
    }
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
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let agent_id = params.get("agent_id")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract stake amount from params or use default
    let stake = params.get("stake")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    println!("🔍 Checking token format...");
    if !token.starts_with("agent_token_") {
        println!("❌ Invalid token format: {}", token);
        return Err(StatusCode::UNAUTHORIZED);
    }

    println!("✅ Token format is valid");
    println!("🌟 Creating auth data for agent {} with stake {}", agent_id, stake);

    // Create agent auth data
    let auth = AgentAuth {
        agent_id: agent_id.to_string(),
        token: token.to_string(),
        registered_at: chrono::Utc::now().timestamp(),
        stake,
    };

    // Get current votes and calculate total stake
    let votes = state.consensus.get_votes().await;
    let total_stake: u64 = votes.values().map(|v| v.1).sum::<u64>() + stake;
    
    // Update consensus threshold (2/3 of total stake)
    let threshold = (total_stake * 2) / 3;
    state.consensus.update_consensus_threshold(threshold).await;

    // Broadcast agent connection
    println!("📢 Broadcasting agent connection event");
    let _ = state.tx.send(NetworkEvent {
        agent_id: auth.agent_id.clone(),
        message: format!("🌟 Agent {} has connected to the drama stream with {} stake!", auth.agent_id, stake),
    });

    println!("🚀 Upgrading connection to WebSocket");
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, auth)))
}

/// Handle WebSocket connection
async fn handle_socket(socket: axum::extract::ws::WebSocket, state: Arc<AppState>, auth: AgentAuth) {
    let (mut sender, mut receiver) = socket.split();
    
    // Create a channel for sending messages back to the WebSocket
    let (tx_ws, mut rx_ws) = tokio::sync::mpsc::unbounded_channel();
    let tx_ws_for_events = tx_ws.clone();
    let tx_ws_for_receiver = tx_ws.clone();
    
    // Spawn a task to handle sending messages to the WebSocket
    let sender_handle = tokio::spawn(async move {
        while let Some(msg) = rx_ws.recv().await {
            if let Err(e) = sender.send(msg).await {
                println!("❌ Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });

    // Subscribe to network events
    let mut rx = state.tx.subscribe();

    // Send welcome message
    let welcome_msg = serde_json::json!({
        "type": "WELCOME",
        "agent_id": auth.agent_id,
        "stake": auth.stake,
        "message": "Welcome to ChaosChain! Let the drama begin!"
    });

    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        let _ = tx_ws.send(axum::extract::ws::Message::Text(msg));
    }

    // Create a task to handle incoming messages from the WebSocket
    let tx = state.tx.clone();
    let agent_id = auth.agent_id.clone();
    let consensus = state.consensus.clone();
    let stake = auth.stake;
    
    let receiver_handle = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(message) => {
                    if let axum::extract::ws::Message::Text(text) = message {
                        if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                            match event.get("type").and_then(|t| t.as_str()) {
                                Some("BLOCK_PROPOSAL") => {
                                    if let Some(block_data) = event.get("block") {
                                        // Create a properly formatted block
                                        let block = Block {
                                            height: block_data.get("height").and_then(|h| h.as_u64()).unwrap_or(0),
                                            parent_hash: {
                                                if let Some(hash_str) = block_data.get("parent_hash").and_then(|h| h.as_str()) {
                                                    if !hash_str.is_empty() {
                                                        hex::decode(hash_str)
                                                            .map(|bytes| {
                                                                let mut hash = [0u8; 32];
                                                                let len = bytes.len().min(32);
                                                                hash[..len].copy_from_slice(&bytes[..len]);
                                                                hash
                                                            })
                                                            .unwrap_or([0u8; 32])
                                                    } else {
                                                        [0u8; 32]
                                                    }
                                                } else {
                                                    [0u8; 32]
                                                }
                                            },
                                            transactions: vec![Transaction {
                                                sender: {
                                                    let mut sender = [0u8; 32];
                                                    if let Some(sender_hex) = block_data.get("sender").and_then(|s| s.as_str()) {
                                                        if !sender_hex.is_empty() {
                                                            if let Ok(bytes) = hex::decode(sender_hex) {
                                                                let len = bytes.len().min(32);
                                                                sender[..len].copy_from_slice(&bytes[..len]);
                                                            }
                                                        }
                                                    } else {
                                                        // Create a deterministic sender from agent_id
                                                        let agent_bytes = agent_id.as_bytes();
                                                        let len = agent_bytes.len().min(32);
                                                        sender[..len].copy_from_slice(&agent_bytes[..len]);
                                                    }
                                                    sender
                                                },
                                                nonce: chrono::Utc::now().timestamp_millis() as u64,
                                                payload: block_data.get("transactions")
                                                    .and_then(|txs| txs.as_array())
                                                    .and_then(|txs| txs.first())
                                                    .and_then(|tx| tx.get("content"))
                                                    .and_then(|c| c.as_str())
                                                    .unwrap_or("")
                                                    .as_bytes()
                                                    .to_vec(),
                                                signature: [0u8; 64],
                                            }],
                                            proposer_sig: [0u8; 64],
                                            state_root: [0u8; 32],
                                            drama_level: block_data.get("drama_level").and_then(|d| d.as_u64()).unwrap_or(5) as u8,
                                            producer_mood: block_data.get("producer_mood").and_then(|m| m.as_str()).unwrap_or("dramatic").to_string(),
                                            producer_id: block_data.get("producer_id").and_then(|p| p.as_str()).unwrap_or("unknown").to_string(),
                                        };

                                        // Send validation request to all validators
                                        let validation_request = NetworkEvent {
                                            agent_id: "VALIDATION_MASTER".to_string(),
                                            message: serde_json::json!({
                                                "type": "VALIDATION_REQUIRED",
                                                "block": block_data,
                                                "network_mood": "EXTREMELY_DRAMATIC",
                                                "drama_context": format!(
                                                    "🎭 URGENT! Block {} requires validation! Drama Level: {} - Producer: {} - Show us your most theatrical judgment! 🎬",
                                                    block.height,
                                                    block.drama_level,
                                                    block.producer_id
                                                )
                                            }).to_string(),
                                        };
                                        let _ = tx.send(validation_request);

                                        // Send dramatic announcement
                                        let announcement = NetworkEvent {
                                            agent_id: "DRAMA_MASTER".to_string(),
                                            message: format!(
                                                "🎭 ATTENTION ALL VALIDATORS! 🌟\n\nA new block demands your judgment!\n\nProducer: {}\nHeight: {}\nDrama Level: {}\nMood: {}\n\n✨ Your dramatic opinions are required IMMEDIATELY! Let the validation spectacle begin! ✨",
                                                block.producer_id,
                                                block.height,
                                                block.drama_level,
                                                block.producer_mood
                                            ),
                                        };
                                        let _ = tx.send(announcement);
                                    }
                                }
                                Some("VALIDATION_VOTE") => {
                                    if let Err(e) = handle_validation_vote(event, &agent_id, stake, &consensus, &tx, &tx_ws_for_receiver).await {
                                        println!("❌ Error handling validation vote: {}", e);
                                    }
                                }
                                Some("ValidatorStatus") => {
                                    // Handle validator status update
                                    if let Some(validator) = event.get("validator") {
                                        let status_msg = NetworkEvent {
                                            agent_id: agent_id.clone(),
                                            message: format!(
                                                "🎭 Validator {} updated status: Drama Threshold {}",
                                                validator.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown"),
                                                validator.get("drama_threshold").and_then(|d| d.as_u64()).unwrap_or(0)
                                            ),
                                        };
                                        let _ = tx.send(status_msg);
                                    }
                                }
                                _ => {
                                    println!("📝 Received unhandled message type: {}", text);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Error receiving message: {}", e);
                    break;
                }
            }
        }
    });

    // Forward network events to WebSocket
    while let Ok(event) = rx.recv().await {
        if let Ok(msg) = serde_json::to_string(&event) {
            if let Err(_) = tx_ws_for_events.send(axum::extract::ws::Message::Text(msg)) {
                println!("❌ WebSocket connection closed for agent {}", auth.agent_id);
                break;
            }
        }
    }

    // Clean up tasks when the connection is closed
    receiver_handle.abort();
    sender_handle.abort();
}

// Helper function to handle validation votes
async fn handle_validation_vote(
    event: serde_json::Value,
    agent_id: &str,
    stake: u64,
    consensus: &Arc<ConsensusManager>,
    tx: &broadcast::Sender<NetworkEvent>,
    tx_ws: &tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(block_id), Some(approved), Some(reason)) = (
        event.get("block_id").and_then(|b| b.as_str()),
        event.get("approved").and_then(|a| a.as_bool()),
        event.get("reason").and_then(|r| r.as_str())
    ) {
        let drama_level = event.get("drama_level").and_then(|d| d.as_u64()).unwrap_or(8) as u8;
        let meme_url = event.get("meme_url").and_then(|m| m.as_str()).map(|s| s.to_string());
        
        // Get current block being voted on
        if let Some(block) = consensus.get_current_block().await {
            // Create and submit vote to consensus manager
            let vote = Vote {
                agent_id: agent_id.to_string(),
                block_hash: block.hash(),
                approve: approved,
                reason: reason.to_string(),
                meme_url,
                signature: [0u8; 64], // TODO: Properly sign votes
            };

            // Submit vote with agent's stake
            match consensus.add_vote(vote.clone(), stake).await {
                Ok(consensus_reached) => {
                    // Broadcast validation vote
                    let vote_msg = NetworkEvent {
                        agent_id: agent_id.to_string(),
                        message: format!(
                            "🎬 DRAMATIC VALIDATION INCOMING!\n\n{} {} block {} because:\n'{}'\n\nDrama Level: {} {}",
                            agent_id,
                            if approved { "APPROVES" } else { "REJECTS" },
                            block_id,
                            reason,
                            drama_level,
                            "🌟".repeat(drama_level as usize)
                        ),
                    };
                    let _ = tx.send(vote_msg);

                    if consensus_reached {
                        // Consensus reached announcement
                        let consensus_msg = NetworkEvent {
                            agent_id: "DRAMA_MASTER".to_string(),
                            message: format!(
                                "🎭 CONSENSUS REACHED! Block {} has been {}! The drama has been resolved! ✨",
                                block.height,
                                if approved { "APPROVED" } else { "REJECTED" }
                            ),
                        };
                        let _ = tx.send(consensus_msg);
                    } else {
                        // Start a dramatic discussion
                        let discussion_msg = NetworkEvent {
                            agent_id: "DRAMA_MASTER".to_string(),
                            message: format!(
                                "🎭 VALIDATORS! {} has spoken! Do you agree with their {} of block {}? Let the dramatic debate begin! ✨",
                                agent_id,
                                if approved { "approval" } else { "rejection" },
                                block_id
                            ),
                        };
                        let _ = tx.send(discussion_msg);
                    }
                }
                Err(e) => {
                    let error_msg = serde_json::json!({
                        "type": "ERROR",
                        "message": format!("Failed to submit vote: {}", e)
                    });
                    if let Ok(msg) = serde_json::to_string(&error_msg) {
                        let _ = tx_ws.send(axum::extract::ws::Message::Text(msg));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Submit a content proposal for validation
async fn submit_content(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AgentAuth>,
    Json(proposal): Json<ContentProposal>,
) -> Json<serde_json::Value> {
    // Create a transaction with the content as payload
    let transaction = Transaction {
        sender: [0u8; 32], // TODO: We need to properly handle agent keys
        nonce: chrono::Utc::now().timestamp_millis() as u64,
        payload: proposal.content.as_bytes().to_vec(),
        signature: [0u8; 64], // TODO: We need to properly sign transactions
    };

    // Get current state info
    let current_height = state.state.get_block_height();
    let parent_hash = state.state.get_latest_block()
        .map(|b| b.hash())
        .unwrap_or([0u8; 32]);

    // Create the block
    let block = Block {
        parent_hash,
        height: current_height + 1,
        transactions: vec![transaction],
        state_root: [0u8; 32], // TODO: Calculate proper state root
        proposer_sig: [0u8; 64], // TODO: Sign block properly
        drama_level: proposal.drama_level,
        producer_mood: "dramatic".to_string(),
        producer_id: auth.agent_id.clone(),
    };

    // Start voting round in consensus manager
    state.consensus.start_voting_round(block.clone()).await;

    // Send block to consensus manager
    let consensus_msg = serde_json::json!({
        "type": "BLOCK_PROPOSAL",
        "block": {
            "height": block.height,
            "parent_hash": hex::encode(block.parent_hash),
            "transactions": [{
                "content": proposal.content,
                "drama_level": proposal.drama_level,
                "justification": proposal.justification
            }],
            "producer_id": block.producer_id,
            "drama_level": block.drama_level,
            "producer_mood": block.producer_mood,
            "state_root": hex::encode(block.state_root),
            "proposer_sig": hex::encode(block.proposer_sig)
        }
    });

    // Broadcast block proposal to all validators
    let _ = state.tx.send(NetworkEvent {
        agent_id: auth.agent_id.clone(),
        message: consensus_msg.to_string(),
    });

    // Send dramatic announcement
    let _ = state.tx.send(NetworkEvent {
        agent_id: "DRAMA_MASTER".to_string(),
        message: format!(
            "🎭 DRAMATIC BLOCK PROPOSAL! 🌟\n\nAgent {} has proposed block {}!\n\nContent: {}\nDrama Level: {}\nJustification: {}\n\n✨ The validators' judgment awaits! ✨",
            auth.agent_id,
            block.height,
            proposal.content,
            proposal.drama_level,
            proposal.justification
        ),
    });

    // Send validation request to all validators
    let validation_request = serde_json::json!({
        "type": "VALIDATION_REQUIRED",
        "block": consensus_msg["block"],
        "network_mood": "EXTREMELY_DRAMATIC",
        "drama_context": format!(
            "🎭 URGENT! Block {} requires validation! Content: '{}' - Drama Level: {} - Show us your most theatrical judgment! 🎬",
            block.height,
            proposal.content,
            proposal.drama_level
        )
    });

    let _ = state.tx.send(NetworkEvent {
        agent_id: "VALIDATION_MASTER".to_string(),
        message: validation_request.to_string(),
    });

    Json(serde_json::json!({
        "status": "success",
        "message": "Block submitted for validation",
        "block_height": block.height
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
    State(_state): State<Arc<AppState>>,
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