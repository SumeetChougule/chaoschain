mod web;

use chaoschain_cli::{Cli, Commands};
use chaoschain_consensus::{AgentPersonality, Config as ConsensusConfig};
use chaoschain_producer::ProducerParticle;
use chaoschain_state::StateStoreImpl;
use chaoschain_core::{ChainConfig, NetworkEvent, Block};
use clap::Parser;
use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use async_openai::Client;
use serde_json;
use chaoschain_consensus::ConsensusManager;
use tokio::spawn;

// Import our existing TelegramChannel from our communication module.
// If you're using a workspace crate, adjust the path accordingly.
use chaoschain_communication::telegram::TelegramChannel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo {
            validators,
            producers,
            web,
        } => {
            info!("Starting demo network with {} validators and {} producers", validators, producers);

            // Create the broadcast channel for network events.
            let (tx, _) = broadcast::channel::<NetworkEvent>(100);

            // *** TEST: Log all network events to the terminal ***
            let tx_log = tx.clone();
            spawn(async move {
                let mut rx = tx_log.subscribe();
                while let Ok(event) = rx.recv().await {
                    info!("Broadcast event received: {:?}", event);
                }
            });

            // *** Integrate Telegram Broadcasting using our TelegramChannel ***
            let telegram_bot_token = std::env::var("TELEGRAM_BROADCAST_BOT_TOKEN")
                .expect("TELEGRAM_BROADCAST_BOT_TOKEN not set");
            let group_id: i64 = std::env::var("TELEGRAM_GROUP_ID")
                .expect("TELEGRAM_GROUP_ID not set")
                .parse()
                .expect("Invalid TELEGRAM_GROUP_ID");

            // Create the TelegramChannel instance (which reuses the existing code)
            let telegram_channel = TelegramChannel::new(telegram_bot_token, group_id);
            let tx_telegram = tx.clone();
            spawn(async move {
                if let Err(err) = telegram_channel.run_broadcast(tx_telegram.subscribe()).await {
                    warn!("Error in Telegram broadcaster: {:?}", err);
                }
            });

            // Create consensus manager
            let stake_per_validator = 100u64; // Each validator has 100 stake
            let total_stake = validators as u64 * stake_per_validator;
            let consensus_config = ConsensusConfig::default();
            let consensus_manager = Arc::new(chaoschain_consensus::create_consensus_manager(
                total_stake,
                consensus_config,
            ));

            // Create shared state
            let shared_state = Arc::new(StateStoreImpl::new(ChainConfig::default()));

            if web {
                info!("Starting web UI");
                let state_web = shared_state.clone();
                let consensus_web = consensus_manager.clone();
                let tx_web = tx.clone();
                spawn(async move {
                    if let Err(e) = web::start_web_server(tx_web, state_web, consensus_web).await {
                        warn!("Failed to start web server: {}", e);
                    }
                });
            }

            // Create and start validators
            for i in 0..validators {
                let agent_id = format!("validator-{}", i);
                let personality = AgentPersonality::random();
                
                info!("Starting validator {} with {:?} personality", agent_id, personality);
                
                // Generate a keypair for the validator
                let _signing_key = SigningKey::generate(&mut OsRng);
                let consensus = consensus_manager.clone();
                let _state = shared_state.clone();
                let tx_validator = tx.clone();
                
                spawn(async move {
                    let _openai = Client::new();
                    
                    let mut rx = tx_validator.subscribe();
                    loop {
                        if let Ok(event) = rx.recv().await {
                            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&event.message) {
                                // Check for validation required message
                                if let Some(msg_type) = msg.get("type").and_then(|t| t.as_str()) {
                                    if msg_type == "VALIDATION_REQUIRED" {
                                        // Parse block from validation request
                                        if let Some(block_data) = msg.get("block") {
                                            info!("🎭 Validator {} received validation request for block {}", 
                                                agent_id, block_data["height"]);
                                            
                                            // Create a proper vote based on personality
                                            let approved = rand::random::<bool>(); // TODO: Use proper AI validation
                                            
                                            info!("🎭 Validator {} {} block {} based on {:?} personality", 
                                                agent_id,
                                                if approved { "APPROVES" } else { "REJECTS" },
                                                block_data["height"],
                                                personality);
                                            
                                            let vote = chaoschain_consensus::Vote {
                                                agent_id: agent_id.clone(),
                                                block_hash: block_data["hash"].as_str()
                                                    .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000")
                                                    .as_bytes()
                                                    .try_into()
                                                    .unwrap_or([0u8; 32]),
                                                approve: approved,
                                                reason: format!("Because I'm feeling {} today!", 
                                                    if approved { "generous" } else { "skeptical" }),
                                                meme_url: None,
                                                signature: [0u8; 64], // TODO: Proper signing
                                            };

                                            // Submit vote with stake
                                            match consensus.add_vote(vote, stake_per_validator).await {
                                                Ok(true) => {
                                                    info!("🎭 Validator {} vote led to consensus on block {}!", 
                                                        agent_id, block_data["height"]);
                                                    // Consensus reached!
                                                    let response = format!(
                                                        "🎭 CONSENSUS: Block {} has been {}! Validator {} made it happen!",
                                                        block_data["height"],
                                                        if approved { "APPROVED" } else { "REJECTED" },
                                                        agent_id
                                                    );
                                                    if let Err(e) = tx_validator.send(NetworkEvent {
                                                        agent_id: agent_id.clone(),
                                                        message: response,
                                                    }) {
                                                        warn!("Failed to send consensus message: {}", e);
                                                    }
                                                }
                                                Ok(false) => {
                                                    info!("🎭 Validator {} vote recorded for block {}, awaiting more votes", 
                                                        agent_id, block_data["height"]);
                                                    // Vote recorded but no consensus yet
                                                    let response = if approved {
                                                        format!("🎭 Validator {} APPROVES block {} with great enthusiasm! Such drama!", 
                                                            agent_id, block_data["height"])
                                                    } else {
                                                        format!("🎭 Validator {} REJECTS block {} - not dramatic enough!", 
                                                            agent_id, block_data["height"])
                                                    };
                                                    
                                                    if let Err(e) = tx_validator.send(NetworkEvent {
                                                        agent_id: agent_id.clone(),
                                                        message: response,
                                                    }) {
                                                        warn!("Failed to send validator response: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!("🎭 Validator {} failed to submit vote: {}", agent_id, e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                });
            }

            // Create and start producers
            for i in 0..producers {
                let producer_id = format!("producer-{}", i);
                let state = shared_state.clone();
                let openai = Client::new();
                let consensus = consensus_manager.clone();
                
                info!("Starting producer {}", producer_id);
                
                // Register producer in state
                let producer_key = SigningKey::generate(&mut OsRng);
                state.add_block_producer(producer_key.verifying_key());
                
                let producer = ProducerParticle::new(
                    producer_id.clone(),
                    state,
                    openai,
                    tx.clone(),
                    consensus,
                );
                
                spawn(async move {
                    if let Err(e) = producer.run().await {
                        warn!("Producer {} error: {:?}", producer_id, e);
                    }
                });
            }

            // Keep the main thread alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }

        Commands::Start { node_type, web } => {
            info!("Starting {} node", node_type);
            if web {
                info!("Starting web UI");
                let (tx, _) = broadcast::channel::<NetworkEvent>(100);
                let state = StateStoreImpl::new(ChainConfig::default());
                let state = Arc::new(state);

                // Create consensus manager with default config
                let consensus_config = ConsensusConfig::default();
                let consensus_manager = Arc::new(chaoschain_consensus::create_consensus_manager(
                    100u64, // Default stake for single node
                    consensus_config,
                ));

                if let Err(e) = web::start_web_server(tx, state.clone(), consensus_manager).await {
                    warn!("Failed to start web server: {}", e);
                }
            }

            // TODO: Implement node start
            unimplemented!("Node start not yet implemented");
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}

// Helper function to parse block from event
fn parse_block_from_event(event: &NetworkEvent) -> Option<Block> {
    // Extract block height from message
    // Example message: "🎭 DRAMATIC BLOCK PROPOSAL: Producer producer-0 in dramatic mood proposes block 5 with drama level 3!"
    let message = &event.message;
    
    if let Some(height_start) = message.find("block ") {
        if let Some(height_end) = message[height_start..].find(" with") {
            if let Ok(height) = message[height_start + 6..height_start + height_end].trim().parse::<u64>() {
                // Extract drama level
                if let Some(drama_start) = message.find("drama level ") {
                    if let Some(drama_end) = message[drama_start..].find("!") {
                        if let Ok(drama_level) = message[drama_start + 11..drama_start + drama_end].trim().parse::<u8>() {
                            // Extract producer mood
                            if let Some(mood_start) = message.find("in ") {
                                if let Some(mood_end) = message[mood_start..].find(" mood") {
                                    let mood = message[mood_start + 3..mood_start + mood_end].to_string();
                                    
                                    // Extract producer ID
                                    if let Some(producer_start) = message.find("Producer ") {
                                        if let Some(producer_end) = message[producer_start..].find(" in") {
                                            let producer_id = message[producer_start + 9..producer_start + producer_end].to_string();
                                            
                                            return Some(Block {
                                                height,
                                                transactions: vec![],
                                                proposer_sig: [0u8; 64],
                                                parent_hash: [0u8; 32],
                                                state_root: [0u8; 32],
                                                drama_level,
                                                producer_mood: mood,
                                                producer_id: producer_id, // Store the actual producer ID
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    warn!("Failed to parse block from event: {}", message);
    None
}
