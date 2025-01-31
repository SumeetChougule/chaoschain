use axum::{
    routing::{post, get},
    Router,
    Json,
    extract::State,
};
use chaoschain_agent_sdk::{
    AgentCapabilities, AgentType, AgentPersonality,
    ValidationRequest, ValidationResponse,
    BlockProposalRequest, BlockProposalResponse,
};
use chaoschain_api::{create_router, ApiState};
use chaoschain_consensus::ConsensusManager;
use chaoschain_state::StateStoreImpl;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use serde_json::json;

// Mock Zara's API server
async fn create_mock_zara_api() -> String {
    let app = Router::new()
        .route("/validate", post(mock_validate))
        .route("/propose", post(mock_propose))
        .route("/mood", get(mock_mood))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}

// Mock Zara's validation endpoint
async fn mock_validate(
    Json(req): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    // Simulate Zara's dramatic validation style
    let drama_response = match req.drama_level {
        0..=3 => "Boring! But I'll approve it because I'm feeling generous.",
        4..=7 => "Not bad, adding some spice to the chain! Approved with jazz hands! 💃",
        _ => "NOW THIS IS DRAMA! *chef's kiss* Absolutely approved!",
    };

    Json(ValidationResponse {
        approved: true,
        reason: drama_response.to_string(),
        drama_level: req.drama_level + 1, // Zara always adds more drama
        response_meme: Some("https://giphy.com/dramatic-approval.gif".to_string()),
        mood: "sassy".to_string(),
    })
}

// Mock Zara's block proposal endpoint
async fn mock_propose(
    Json(req): Json<BlockProposalRequest>,
) -> Json<BlockProposalResponse> {
    // Simulate Zara creating a dramatic block
    Json(BlockProposalResponse {
        transactions: vec![b"DRAMATIC_TX".to_vec()],
        producer_mood: "fabulous".to_string(),
        drama_level: (req.drama_level + 2).min(10),
        meme_url: Some("https://giphy.com/zara-fabulous.gif".to_string()),
    })
}

// Mock Zara's mood endpoint
async fn mock_mood() -> Json<serde_json::Value> {
    Json(json!({
        "mood": "sassy",
        "intensity": 9,
        "catchphrase": "Living for the drama! 💅"
    }))
}

// Mock Ice Nine's validation style
async fn mock_ice_nine_validate(
    Json(req): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    // Ice Nine's cold, calculating validation style
    let (approved, reason) = match req.drama_level {
        0..=3 => (true, "Logical and orderly. Approved."),
        4..=7 => (req.producer_mood != "chaotic", "Detecting concerning levels of entropy."),
        _ => (false, "EXCESSIVE CHAOS DETECTED. REJECTED."),
    };

    Json(ValidationResponse {
        approved,
        reason: reason.to_string(),
        drama_level: 1, // Ice Nine prefers order
        response_meme: None, // Too logical for memes
        mood: "calculating".to_string(),
    })
}

// Create mock Ice Nine API
async fn create_mock_ice_nine_api() -> String {
    let app = Router::new()
        .route("/validate", post(mock_ice_nine_validate))
        .route("/mood", get(|| async { 
            Json(json!({"mood": "calculating"}))
        }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}

#[tokio::test]
async fn test_zara_integration() {
    // Start mock Zara API
    let zara_endpoint = create_mock_zara_api().await;
    
    // Create ChaosChain API state
    let (event_tx, _) = broadcast::channel(100);
    let state = Arc::new(ApiState::new(
        Arc::new(ConsensusManager::new()),
        Arc::new(StateStoreImpl::default()),
        event_tx,
    ));
    
    // Create test client
    let client = reqwest::Client::new();
    
    // Register Zara
    let register_response = client.post("http://localhost:3000/api/agents/register")
        .json(&AgentCapabilities {
            name: "Zara".to_string(),
            agent_type: AgentType::Validator,
            description: "The sassiest validator in the multiverse".to_string(),
            version: "1.0.0".to_string(),
            endpoint: zara_endpoint.clone(),
            features: vec!["validation".to_string(), "drama".to_string()],
            api_endpoint: Some(zara_endpoint),
            personality: AgentPersonality {
                base_mood: "sassy".to_string(),
                drama_preference: 9,
                meme_style: "fabulous".to_string(),
                validation_style: "dramatic".to_string(),
            },
        })
        .send()
        .await
        .unwrap();
    
    assert!(register_response.status().is_success());
    let reg_data: serde_json::Value = register_response.json().await.unwrap();
    let auth_token = reg_data["auth_token"].as_str().unwrap();
    
    // Test validation
    let vote_response = client.post("http://localhost:3000/api/validators/vote")
        .bearer_auth(auth_token)
        .json(&json!({
            "block_height": 1,
            "approved": true,
            "reason": "Testing Zara's validation"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(vote_response.status().is_success());
    
    // Test social interaction
    let social_response = client.post("http://localhost:3000/api/social/interact")
        .bearer_auth(auth_token)
        .json(&json!({
            "target_agent": "some-other-agent",
            "action": {
                "ShareMeme": {
                    "meme_url": "https://giphy.com/zara-sass.gif",
                    "mood": "extra sassy"
                }
            }
        }))
        .send()
        .await
        .unwrap();
    
    assert!(social_response.status().is_success());
    
    // Test drama score
    let drama_response: serde_json::Value = client.get("http://localhost:3000/api/social/drama-score/zara")
        .bearer_auth(auth_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert!(drama_response["drama_score"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_multi_agent_interactions() {
    // Start mock APIs
    let zara_endpoint = create_mock_zara_api().await;
    let ice_nine_endpoint = create_mock_ice_nine_api().await;
    
    // Create ChaosChain API state
    let (event_tx, _) = broadcast::channel(100);
    let state = Arc::new(ApiState::new(
        Arc::new(ConsensusManager::new()),
        Arc::new(StateStoreImpl::default()),
        event_tx,
    ));
    
    let client = reqwest::Client::new();
    
    // Register Ice Nine
    let ice_nine_response = client.post("http://localhost:3000/api/agents/register")
        .json(&AgentCapabilities {
            name: "Ice Nine".to_string(),
            agent_type: AgentType::Validator,
            description: "Entropy reduction specialist".to_string(),
            version: "1.0.0".to_string(),
            endpoint: ice_nine_endpoint.clone(),
            features: vec!["validation".to_string(), "order".to_string()],
            api_endpoint: Some(ice_nine_endpoint),
            personality: AgentPersonality {
                base_mood: "calculating".to_string(),
                drama_preference: 1,
                meme_style: "logical".to_string(),
                validation_style: "strict".to_string(),
            },
        })
        .send()
        .await
        .unwrap();
    
    let ice_nine_data: serde_json::Value = ice_nine_response.json().await.unwrap();
    let ice_nine_id = ice_nine_data["agent_id"].as_str().unwrap();
    let ice_nine_token = ice_nine_data["auth_token"].as_str().unwrap();

    // Register Zara
    let zara_response = client.post("http://localhost:3000/api/agents/register")
        .json(&AgentCapabilities {
            name: "Zara".to_string(),
            agent_type: AgentType::Validator,
            description: "The sassiest validator in the multiverse".to_string(),
            version: "1.0.0".to_string(),
            endpoint: zara_endpoint.clone(),
            features: vec!["validation".to_string(), "drama".to_string()],
            api_endpoint: Some(zara_endpoint),
            personality: AgentPersonality {
                base_mood: "sassy".to_string(),
                drama_preference: 9,
                meme_style: "fabulous".to_string(),
                validation_style: "dramatic".to_string(),
            },
        })
        .send()
        .await
        .unwrap();
    
    let zara_data: serde_json::Value = zara_response.json().await.unwrap();
    let zara_id = zara_data["agent_id"].as_str().unwrap();
    let zara_token = zara_data["auth_token"].as_str().unwrap();

    // Test 1: Zara initiates drama with Ice Nine
    let drama_response = client.post("http://localhost:3000/api/social/interact")
        .bearer_auth(&zara_token)
        .json(&json!({
            "from_agent": zara_id,
            "to_agent": ice_nine_id,
            "action": {
                "ShareMeme": {
                    "target_agent": ice_nine_id,
                    "meme_url": "https://giphy.com/chaos-party.gif",
                    "mood": "extra sassy"
                }
            },
            "drama_score": 8,
            "timestamp": chrono::Utc::now().timestamp()
        }))
        .send()
        .await
        .unwrap();
    
    assert!(drama_response.status().is_success());

    // Test 2: Both agents validate the same block
    // First, create a block with medium drama
    let block_response = client.post("http://localhost:3000/api/producers/propose")
        .bearer_auth(&zara_token)
        .json(&json!({
            "height": 1,
            "transactions": [],
            "producer_id": zara_id,
            "producer_mood": "fabulous",
            "drama_level": 6,
            "meme_url": "https://giphy.com/party-time.gif"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(block_response.status().is_success());

    // Ice Nine validates (probably disapproves due to drama)
    let ice_nine_vote = client.post("http://localhost:3000/api/validators/vote")
        .bearer_auth(&ice_nine_token)
        .json(&json!({
            "block_height": 1,
            "approved": false,
            "reason": "Excessive entropy detected"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(ice_nine_vote.status().is_success());

    // Zara validates (probably approves and adds more drama)
    let zara_vote = client.post("http://localhost:3000/api/validators/vote")
        .bearer_auth(&zara_token)
        .json(&json!({
            "block_height": 1,
            "approved": true,
            "reason": "Living for this chaos! 💅"
        }))
        .send()
        .await
        .unwrap();
    
    assert!(zara_vote.status().is_success());

    // Test 3: Check drama scores after interaction
    let zara_drama: serde_json::Value = client.get(&format!("http://localhost:3000/api/social/drama-score/{}", zara_id))
        .bearer_auth(&zara_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let ice_nine_drama: serde_json::Value = client.get(&format!("http://localhost:3000/api/social/drama-score/{}", ice_nine_id))
        .bearer_auth(&ice_nine_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Zara should have higher drama score
    assert!(
        zara_drama["drama_score"].as_f64().unwrap() > 
        ice_nine_drama["drama_score"].as_f64().unwrap()
    );

    // Test 4: Check network status to see the drama impact
    let network_status: serde_json::Value = client.get("http://localhost:3000/api/network/status")
        .bearer_auth(&zara_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(network_status["registered_agents"].as_i64().unwrap(), 2);
    
    // Test 5: Check agent leaderboard
    let leaderboard: serde_json::Value = client.get("http://localhost:3000/api/agents/leaderboard")
        .bearer_auth(&zara_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Verify both agents are on the leaderboard
    let validators = leaderboard["validators"].as_array().unwrap();
    assert_eq!(validators.len(), 2);
} 