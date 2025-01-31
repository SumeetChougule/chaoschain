use axum::{
    routing::get,
    Router,
    extract::State,
    response::{Html, Sse, IntoResponse},
    Json,
};
use futures::StreamExt;
use serde::Serialize;
use std::{sync::Arc, net::SocketAddr, convert::Infallible};
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
use std::time::Duration;

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
    pub latest_token_insights: Vec<String>,
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

/// Initial market analysis from Zara
const INITIAL_MARKET_ANALYSIS: &str = r#"🔍 TOKEN INSIGHT: $BULLY showing a bearish trend, priced at $0.0397 (-9.59% in 24h). RSI at 38.86 suggests oversold territory, while negative MACD reinforces downward momentum. Key support at $0.0374, backed by high volume (7,137,170.03)—watch price hold above this level for potential reversal signals. Overall risk remains high with a negative return/risk ratio (-0.16)."#;

/// Start the web server
pub async fn start_web_server(tx: broadcast::Sender<NetworkEvent>, state: Arc<StateStoreImpl>) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState {
        tx: tx.clone(),
        state: state.clone(),
    });

    // Send initial market analysis from Zara
    let _ = tx.send(NetworkEvent {
        agent_id: "zara".to_string(),
        message: INITIAL_MARKET_ANALYSIS.to_string(),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/events", get(event_stream))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Web UI available at http://localhost:3000");
    
    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app
    ).await?;

    Ok(())
}

/// Get network status including latest blocks and token insights
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
    
    // For demo, we'll just show the latest token insight
    let latest_token_insights = vec![
        "BULLY: $0.0397 (-9.59%) | RSI: 38.86 | Volume: 7.14M | Risk: -0.16".to_string()
    ];

    Json(NetworkStatus {
        validator_count: 4, // We know we started with 4 validators
        producer_count: chain_state.producers.len() as u32,
        latest_block,
        total_blocks_produced: latest_block,
        total_blocks_validated: latest_block,
        latest_blocks,
        latest_token_insights,
    })
}

/// Stream network events to the web UI
async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg| {
            let msg = msg.unwrap();
            Ok::<_, Infallible>(axum::response::sse::Event::default().data(
                serde_json::json!({
                    "agent_id": msg.agent_id,
                    "message": msg.message,
                })
                .to_string(),
            ))
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn index() -> Html<String> {
    // Pre-format the initial analysis for immediate display
    let initial_metrics = [
        ("Price", "$0.0397"),
        ("24h Change", "-9.59%"),
        ("RSI", "38.86"),
        ("Volume", "7,137,170.03"),
        ("Support", "$0.0374"),
        ("Risk Ratio", "-0.16")
    ];

    let initial_metrics_html = initial_metrics
        .iter()
        .map(|(label, value)| format!(
            r#"<div class="metric">
                <div style="font-size: 0.9em; opacity: 0.8">{}</div>
                <div style="font-size: 1.2em; font-weight: bold">{}</div>
            </div>"#,
            label, value
        ))
        .collect::<Vec<_>>()
        .join("");

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Zara's ChaosChain Oracle Feed</title>
            <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&display=swap">
            <style>
                :root {{
                    --neon-blue: #00f3ff;
                    --neon-purple: #9d00ff;
                    --neon-pink: #ff00f7;
                    --dark-bg: #0a0b0e;
                    --card-bg: #13141f;
                    --text-glow: 0 0 10px rgba(0, 243, 255, 0.5);
                }}
                
                body {{ 
                    font-family: 'Space Mono', monospace;
                    margin: 0;
                    padding: 40px;
                    line-height: 1.6;
                    background: var(--dark-bg);
                    color: #fff;
                    background-image: 
                        radial-gradient(circle at 20% 20%, rgba(157, 0, 255, 0.1) 0%, transparent 50%),
                        radial-gradient(circle at 80% 80%, rgba(0, 243, 255, 0.1) 0%, transparent 50%);
                }}

                .container {{
                    max-width: 800px;
                    margin: 0 auto;
                }}

                .title-section {{
                    text-align: center;
                    margin-bottom: 40px;
                    position: relative;
                }}

                .main-title {{
                    font-size: 2.5em;
                    font-weight: bold;
                    margin: 0;
                    background: linear-gradient(45deg, var(--neon-blue), var(--neon-purple));
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    text-shadow: var(--text-glow);
                    position: relative;
                }}

                .subtitle {{
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 1.1em;
                    margin-top: 10px;
                }}

                .zara-post {{ 
                    background: linear-gradient(135deg, #1a1b27, #2a1b3d);
                    color: white;
                    padding: 25px;
                    border-radius: 12px;
                    margin-bottom: 30px;
                    box-shadow: 0 8px 32px rgba(0, 243, 255, 0.1);
                    border: 1px solid rgba(157, 0, 255, 0.2);
                    backdrop-filter: blur(10px);
                    position: relative;
                    overflow: hidden;
                }}

                .zara-post::before {{
                    content: '';
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    height: 1px;
                    background: linear-gradient(90deg, transparent, var(--neon-purple), transparent);
                }}

                .zara-header {{
                    display: flex;
                    align-items: center;
                    margin-bottom: 20px;
                    position: relative;
                }}

                .zara-avatar {{
                    width: 60px;
                    height: 60px;
                    border-radius: 50%;
                    margin-right: 15px;
                    background: linear-gradient(135deg, rgba(157, 0, 255, 0.2), rgba(0, 243, 255, 0.2));
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 30px;
                    border: 2px solid rgba(157, 0, 255, 0.3);
                    box-shadow: 0 0 20px rgba(157, 0, 255, 0.2);
                }}

                .zara-name {{
                    font-size: 1.4em;
                    font-weight: bold;
                    color: var(--neon-blue);
                    text-shadow: var(--text-glow);
                }}

                .zara-title {{
                    font-size: 0.9em;
                    color: rgba(255, 255, 255, 0.7);
                }}

                .zara-analysis {{
                    background: rgba(19, 20, 31, 0.8);
                    padding: 20px;
                    border-radius: 8px;
                    margin: 15px 0;
                    line-height: 1.6;
                    font-size: 1.1em;
                    border: 1px solid rgba(0, 243, 255, 0.1);
                }}

                .metrics {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
                    gap: 15px;
                    margin-top: 15px;
                }}

                .metric {{
                    background: rgba(19, 20, 31, 0.8);
                    padding: 15px;
                    border-radius: 8px;
                    transition: all 0.3s ease;
                    border: 1px solid rgba(0, 243, 255, 0.1);
                    position: relative;
                    overflow: hidden;
                }}

                .metric:hover {{
                    transform: translateY(-2px);
                    border-color: var(--neon-blue);
                    box-shadow: 0 0 20px rgba(0, 243, 255, 0.2);
                }}

                .metric::before {{
                    content: '';
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    background: linear-gradient(45deg, transparent, rgba(0, 243, 255, 0.1), transparent);
                    transform: translateX(-100%);
                    transition: transform 0.5s;
                }}

                .metric:hover::before {{
                    transform: translateX(100%);
                }}

                .token-insight-votes {{ 
                    background: rgba(19, 20, 31, 0.8);
                    padding: 15px;
                    margin: 10px 0;
                    border-radius: 8px;
                    border-left: 4px solid var(--neon-purple);
                    box-shadow: 0 4px 20px rgba(157, 0, 255, 0.1);
                }}

                .token-insight-consensus {{ 
                    font-weight: bold;
                    margin: 15px 0;
                    padding: 15px;
                    background: rgba(19, 20, 31, 0.8);
                    border-radius: 8px;
                    text-align: center;
                    border: 1px solid var(--neon-blue);
                    text-shadow: var(--text-glow);
                }}

                .block-events {{ 
                    background: rgba(19, 20, 31, 0.8);
                    padding: 15px;
                    border-radius: 8px;
                    margin: 5px 0;
                    border: 1px solid rgba(0, 243, 255, 0.1);
                }}

                .consensus {{ color: var(--neon-blue); font-weight: bold; }}
                .reject {{ color: var(--neon-pink); }}
                .approve {{ color: #00ff9d; }}
                .timestamp {{ color: rgba(255, 255, 255, 0.5); font-size: 0.8em; }}

                .section-title {{
                    margin-top: 30px;
                    padding-bottom: 10px;
                    border-bottom: 2px solid rgba(0, 243, 255, 0.2);
                    color: var(--neon-blue);
                    text-shadow: var(--text-glow);
                    font-size: 1.5em;
                }}

                .agent-discussion {{
                    background: rgba(19, 20, 31, 0.8);
                    padding: 20px;
                    border-radius: 12px;
                    margin: 20px 0;
                    border: 1px solid rgba(157, 0, 255, 0.2);
                    box-shadow: 0 8px 32px rgba(157, 0, 255, 0.1);
                }}

                .agent-discussion h3 {{
                    color: var(--neon-purple);
                    text-shadow: 0 0 10px rgba(157, 0, 255, 0.5);
                    margin-top: 0;
                }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="title-section">
                    <h1 class="main-title">🔮 Zara's ChaosChain Oracle</h1>
                    <div class="subtitle">Where Market Analysis Meets Chaos Theory</div>
                </div>
                
                <div class="zara-post">
                    <div class="zara-header">
                        <div class="zara-avatar">🔮</div>
                        <div>
                            <div class="zara-name">Zara</div>
                            <div class="zara-title">Chaos Market Oracle</div>
                        </div>
                    </div>
                    <div id="token-insight">
                        <div class="zara-analysis">
                            {analysis}
                        </div>
                        <div class="metrics">
                            {metrics}
                        </div>
                    </div>
                </div>

                <div class="agent-discussion">
                    <h3>🤖 Agent Discussion on Zara's Analysis</h3>
                    <div id="token-votes"></div>
                    <div id="token-consensus"></div>
                </div>
                
                <h2 class="section-title">📊 Block Production & Validation</h2>
                <div id="events"></div>
            </div>

            <script>
                const tokenInsightDiv = document.getElementById('token-insight');
                const tokenVotesDiv = document.getElementById('token-votes');
                const tokenConsensusDiv = document.getElementById('token-consensus');
                const eventsDiv = document.getElementById('events');
                let latestTokenInsight = null;
                let tokenVotes = [];
                let blockEvents = [];

                function formatTokenInsight(message) {{
                    // Extract metrics from the message
                    const metrics = [
                        {{ label: "Price", value: message.match(/\$[\d.]+/)[0] }},
                        {{ label: "24h Change", value: message.match(/\(([-\d.]+%)/)[1] }},
                        {{ label: "RSI", value: message.match(/RSI at ([\d.]+)/)[1] }},
                        {{ label: "Volume", value: message.match(/volume \(([\d,.]+)/)[1] }},
                        {{ label: "Support", value: message.match(/support at \$([\d.]+)/)[1] }},
                        {{ label: "Risk Ratio", value: message.match(/risk ratio \(([-\d.]+)/)[1] }}
                    ];

                    return `
                        <div class="zara-analysis">
                            ${{message.replace('🔍 TOKEN INSIGHT: ', '')}}
                        </div>
                        <div class="metrics">
                            ${{metrics.map(m => 
                                `<div class="metric">
                                    <div style="font-size: 0.9em; opacity: 0.8">${{m.label}}</div>
                                    <div style="font-size: 1.2em; font-weight: bold">${{m.value}}</div>
                                </div>`
                            ).join('')}}
                        </div>
                    `;
                }}

                const eventSource = new EventSource('/events');
                eventSource.onmessage = (event) => {{
                    const data = JSON.parse(event.data);
                    const message = data.message;
                    const timestamp = new Date().toLocaleTimeString();

                    if (message.startsWith('🔍 TOKEN INSIGHT:')) {{
                        latestTokenInsight = message;
                        tokenInsightDiv.innerHTML = formatTokenInsight(message);
                    }} else if (message.includes('VALIDATES') || message.includes('REJECTS')) {{
                        if (message.includes('block')) {{
                            // Block validation event
                            const eventHtml = `
                                <div class="event block-events">
                                    <span class="timestamp">${{timestamp}}</span>
                                    <span class="${{message.includes('REJECTS') ? 'reject' : 'approve'}}">
                                        ${{message}}
                                    </span>
                                </div>
                            `;
                            blockEvents.unshift(eventHtml);
                            eventsDiv.innerHTML = blockEvents.join('');
                        }} else {{
                            // Token insight validation
                            const voteHtml = `
                                <div class="token-insight-votes">
                                    <span class="timestamp">${{timestamp}}</span>
                                    <span class="${{message.includes('VALIDATES') ? 'approve' : 'reject'}}">
                                        ${{message}}
                                    </span>
                                </div>
                            `;
                            tokenVotes.unshift(voteHtml);
                            tokenVotesDiv.innerHTML = tokenVotes.join('');

                            // Check for consensus
                            const validCount = tokenVotes.filter(v => v.includes('VALIDATES')).length;
                            const rejectCount = tokenVotes.filter(v => v.includes('REJECTS')).length;
                            if (validCount >= 3) {{
                                tokenConsensusDiv.innerHTML = `
                                    <div class="token-insight-consensus">
                                        ✨ CONSENSUS REACHED: The majority of agents have validated Zara's market analysis!
                                    </div>
                                `;
                            }} else if (rejectCount >= 3) {{
                                tokenConsensusDiv.innerHTML = `
                                    <div class="token-insight-consensus reject">
                                        ❌ CONSENSUS REACHED: The majority of agents have rejected Zara's market analysis!
                                    </div>
                                `;
                            }}
                        }}
                    }} else if (message.includes('DRAMATIC BLOCK PROPOSAL')) {{
                        const eventHtml = `
                            <div class="event block-events">
                                <span class="timestamp">${{timestamp}}</span>
                                ${{message}}
                            </div>
                        `;
                        blockEvents.unshift(eventHtml);
                        eventsDiv.innerHTML = blockEvents.join('');
                    }}
                }};
            </script>
        </body>
        </html>
        "#,
        analysis = INITIAL_MARKET_ANALYSIS.replace("🔍 TOKEN INSIGHT: ", ""),
        metrics = initial_metrics_html
    ))
} 