mod web;

use chaoschain_cli::{Cli, Commands};
use chaoschain_consensus::{AgentPersonality, Config as ConsensusConfig};
use chaoschain_producer::ProducerParticle;
use chaoschain_state::{StateStore, StateStoreImpl};
use chaoschain_core::{ChainConfig, NetworkEvent, Block, TokenInsight};
use clap::Parser;
use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use async_openai::{
    Client,
    types::{
        CreateChatCompletionRequestArgs,
        ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestMessage,
        Role,
    }
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    let subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo {
            validators,
            producers,
            web,
            token_symbol,
            price,
            price_change,
            rsi,
            volume,
            support,
            risk,
            sentiment,
            chart,
            agent_id,
        } => {
            info!("Starting demo network with {} validators and {} producers", validators, producers);

            let (tx, _) = broadcast::channel(100);
            let web_tx = tx.clone();

            // Create consensus manager
            let stake_per_validator = 100u64;
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
                let state = shared_state.clone();
                tokio::spawn(async move {
                    web::start_web_server(web_tx, state).await.unwrap();
                });
            }

            // Create and start validators
            for i in 0..validators {
                let agent_id = format!("validator-{}", i);
                let personality = AgentPersonality::random();
                
                info!("Starting validator {} with {:?} personality", agent_id, personality);
                
                let signing_key = SigningKey::generate(&mut OsRng);
                let tx = tx.clone();
                let agent_id_clone = agent_id.clone();
                let rx = tx.subscribe();
                let consensus = consensus_manager.clone();
                let state = shared_state.clone();
                
                tokio::spawn(async move {
                    let openai = Client::new();
                    let mut rx = rx;
                    
                    loop {
                        if let Ok(event) = rx.recv().await {
                            // Handle token insights
                            if let Some(insight) = parse_token_insight_from_event(&event) {
                                let prompt = format!(
                                    "You are validator {} with a {} personality.\n\
                                    A market analyst has shared a token insight for ${}:\n\
                                    Price: ${} ({}% in 24h)\n\
                                    RSI: {}\n\
                                    Volume: {}\n\
                                    Support level: ${}\n\
                                    Risk ratio: {}\n\
                                    Sentiment: {}\n\n\
                                    Based on your personality and these metrics, validate this insight.\n\
                                    Consider technical indicators, market sentiment, and your own style.\n\
                                    Respond with VALID or INVALID and explain your reasoning dramatically!",
                                    agent_id_clone,
                                    personality.to_string(),
                                    insight.token_symbol,
                                    insight.price,
                                    insight.price_change_24h,
                                    insight.rsi,
                                    insight.volume,
                                    insight.support_level,
                                    insight.risk_ratio,
                                    insight.sentiment
                                );

                                let message = match ChatCompletionRequestUserMessageArgs::default()
                                    .content(prompt)
                                    .build() {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            warn!("Failed to build message: {}", e);
                                            continue;
                                        }
                                    };

                                let request = match CreateChatCompletionRequestArgs::default()
                                    .model("gpt-4-turbo-preview")
                                    .messages(vec![ChatCompletionRequestMessage::User(message)])
                                    .temperature(0.9)
                                    .max_tokens(200u16)
                                    .build() {
                                        Ok(req) => req,
                                        Err(e) => {
                                            warn!("Failed to build request: {}", e);
                                            continue;
                                        }
                                    };

                                if let Ok(response) = openai.chat().create(request).await {
                                    if let Some(choice) = response.choices.first() {
                                        if let Some(content) = &choice.message.content {
                                            let approve = content.to_lowercase().contains("valid");
                                            let drama = format!(
                                                "🎭 {} {}: {}",
                                                agent_id_clone,
                                                if approve { "VALIDATES" } else { "REJECTS" },
                                                content
                                            );
                                            let _ = tx.send(NetworkEvent {
                                                agent_id: agent_id_clone.clone(),
                                                message: drama,
                                            });
                                        }
                                    }
                                }
                            }
                            
                            // React to block proposals based on personality
                            if event.message.contains("DRAMATIC BLOCK PROPOSAL") {
                                // Parse block from event message
                                if let Some(block) = parse_block_from_event(&event) {
                                    // Create a proper vote
                                    let vote = chaoschain_consensus::Vote {
                                        agent_id: agent_id_clone.clone(),
                                        block_hash: block.hash(),
                                        approve: rand::random::<bool>(),
                                        reason: "Because I felt like it!".to_string(),
                                        meme_url: None,
                                        signature: [0u8; 64], // TODO: Proper signing
                                    };

                                    // Store vote approval before moving
                                    let approved = vote.approve;

                                    // Submit vote with stake
                                    match consensus.add_vote(vote, stake_per_validator).await {
                                        Ok(true) => {
                                            // Consensus reached!
                                            let response = format!(
                                                "🎭 CONSENSUS: Block {} has been {}! Validator {} made it happen!",
                                                block.height,
                                                if approved { "APPROVED" } else { "REJECTED" },
                                                agent_id_clone
                                            );
                                            if let Err(e) = tx.send(NetworkEvent {
                                                agent_id: agent_id_clone.clone(),
                                                message: response,
                                            }) {
                                                warn!("Failed to send consensus message: {}", e);
                                            }

                                            // Store block in state if approved
                                            if approved {
                                                info!("Storing block {} in state", block.height);
                                                if let Err(e) = state.apply_block(&block) {
                                                    warn!("Failed to store block: {}", e);
                                                }
                                            }
                                        }
                                        Ok(false) => {
                                            // Vote recorded but no consensus yet
                                            let response = if approved {
                                                format!("🎭 Validator {} APPROVES block {} with great enthusiasm! Such drama!", agent_id_clone, block.height)
                                            } else {
                                                format!("🎭 Validator {} REJECTS block {} - not dramatic enough!", agent_id_clone, block.height)
                                            };
                                            
                                            if let Err(e) = tx.send(NetworkEvent {
                                                agent_id: agent_id_clone.clone(),
                                                message: response,
                                            }) {
                                                warn!("Failed to send validator response: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to submit vote: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                });
            }

            // Create a token insight
            let insight = TokenInsight {
                token_symbol: token_symbol.unwrap_or_else(|| "BULLY".to_string()),
                price: price.unwrap_or(0.0397),
                price_change_24h: price_change.unwrap_or(-9.59),
                rsi: rsi.unwrap_or(38.86),
                volume: volume.unwrap_or(7137170.03),
                support_level: support.unwrap_or(0.0374),
                risk_ratio: risk.unwrap_or(-0.16),
                sentiment: sentiment.unwrap_or_else(|| "bearish".to_string()),
                chart_url: chart.unwrap_or_else(|| "chart.png".to_string()),
                timestamp: chrono::Utc::now().timestamp() as u64,
            };

            // Broadcast the insight
            tx.send(NetworkEvent {
                agent_id: agent_id.unwrap_or_else(|| "market-analyst".to_string()),
                message: format!(
                    "🔍 TOKEN INSIGHT: ${} is showing a {} trend, priced at ${} ({}% in 24h). RSI at {} suggests {}. Key support at ${}, backed by volume of {}. Risk ratio: {}",
                    insight.token_symbol,
                    insight.sentiment,
                    insight.price,
                    insight.price_change_24h,
                    insight.rsi,
                    if insight.rsi < 30.0 { "oversold territory" } else if insight.rsi > 70.0 { "overbought territory" } else { "neutral territory" },
                    insight.support_level,
                    insight.volume,
                    insight.risk_ratio
                ),
            })?;

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
                
                tokio::spawn(async move {
                    producer.run().await.unwrap();
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
                let (tx, _) = tokio::sync::broadcast::channel(100);
                let state = StateStoreImpl::new(ChainConfig::default());
                let state = Arc::new(state);
                if let Err(e) = web::start_web_server(tx, state.clone()).await {
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

// Helper function to parse token insight from event
fn parse_token_insight_from_event(event: &NetworkEvent) -> Option<TokenInsight> {
    if event.message.starts_with("🔍 TOKEN INSIGHT:") {
        // In a real implementation, we would parse the actual values from the message
        // For demo, we'll return a sample insight
        Some(TokenInsight {
            token_symbol: event.message.split('$').nth(1)?.split(' ').next()?.to_string(),
            price: 0.0397,
            price_change_24h: -9.59,
            rsi: 38.86,
            volume: 7137170.03,
            support_level: 0.0374,
            risk_ratio: -0.16,
            sentiment: "bearish".to_string(),
            chart_url: "chart.png".to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
        })
    } else {
        None
    }
}
