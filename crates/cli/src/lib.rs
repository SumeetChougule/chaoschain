use chaoschain_core::{Block, Transaction};
use chaoschain_state::StateStore;
use chaoschain_consensus::{Agent, Config as ConsensusConfig};
use chaoschain_p2p::{Config as P2PConfig};
use chaoschain_producer::{ProducerConfig, Producer};
use chaoschain_bridge::{Config as BridgeConfig};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory
    pub data_dir: String,
    /// OpenAI API key
    pub openai_api_key: String,
    /// Ethereum RPC URL
    pub eth_rpc: String,
    /// Web UI port
    pub web_port: u16,
}

/// CLI commands
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Run a demo network
    Demo {
        /// Number of validator agents
        #[arg(long, default_value = "4")]
        validators: u32,

        /// Number of block producers
        #[arg(long, default_value = "1")]
        producers: u32,

        /// Enable web UI
        #[arg(long)]
        web: bool,

        /// Token symbol for insight demo
        #[arg(long)]
        token_symbol: Option<String>,

        /// Token price
        #[arg(long)]
        price: Option<f64>,

        /// 24h price change percentage
        #[arg(long)]
        price_change: Option<f64>,

        /// RSI value
        #[arg(long)]
        rsi: Option<f64>,

        /// Trading volume
        #[arg(long)]
        volume: Option<f64>,

        /// Support level
        #[arg(long)]
        support: Option<f64>,

        /// Risk ratio
        #[arg(long)]
        risk: Option<f64>,

        /// Market sentiment
        #[arg(long)]
        sentiment: Option<String>,

        /// Chart URL
        #[arg(long)]
        chart: Option<String>,

        /// Agent ID for token insight
        #[arg(long)]
        agent_id: Option<String>,
    },
    
    /// Start a node
    Start {
        /// Node type (validator/producer)
        #[arg(long, default_value = "validator")]
        node_type: String,
        
        /// Start web UI
        #[arg(long)]
        web: bool,
    },
} 