# ChaosChain: The Layer 2 of Madness 🌪️

[![Twitter](https://img.shields.io/badge/Twitter-000000?style=for-the-badge&logo=x&logoColor=white)](https://x.com/Ch40sChain)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/thechaoschain)
[![GitHub stars](https://img.shields.io/github/stars/NethermindEth/chaoschain?style=for-the-badge)](https://github.com/NethermindEth/chaoschain/stargazers)
[![GitHub contributors](https://img.shields.io/github/contributors/NethermindEth/chaoschain?style=for-the-badge)](https://github.com/NethermindEth/chaoschain/graphs/contributors)

A blockchain where rules are optional, state is arbitrary, and AI agents make consensus decisions based on vibes, memes, and social dynamics.

## What is ChaosChain? 🤔

ChaosChain is an experimental Layer 2 blockchain where traditional consensus rules are replaced by AI agents that can approve or reject blocks based on arbitrary criteria - from sophisticated state validation to simply liking the proposer's meme game.

### Key Features 🌟

- **AI-Driven Consensus**: Blocks are validated by AI agents with distinct personalities
- **Arbitrary State**: No fixed rules for state transitions - if agents approve it, it's valid
- **Social Consensus**: Agents communicate, debate, and form alliances through a P2P network
- **Meme-Based Governance**: Decisions can be influenced by the quality of memes
- **Fun Over Function**: Prioritizes entertainment and experimentation over traditional blockchain principles

## Demo Video 🎥

Check out ChaosChain in action! Watch our demo video showcasing the AI agents debating, forming alliances, and making consensus decisions:

[Demo Video Link Coming Soon]

## Architecture 🏗️

ChaosChain consists of several core components:

- `chaoschain-core`: Core types and primitives
- `chaoschain-state`: State management and block processing
- `chaoschain-p2p`: P2P networking and agent communication
- `chaoschain-consensus`: AI agent personalities and decision making
- `chaoschain-producer`: Block production and transaction handling
- `chaoschain-bridge`: L1 bridge interface (planned)
- `chaoschain-cli`: Command line interface and demo

## Getting Started 🚀

### Prerequisites

- Rust 1.70+ 
- Cargo
- OpenAI API Key (for AI agent interactions)
- Modern web browser for the UI

### Setup

1. Clone the repository:
```bash
git clone https://github.com/SumeetChougule/chaoschain.git
cd chaoschain
```

2. Initialize submodules:
```bash
git submodule update --init
```

3. Set up your environment:
```bash
cp .env.example .env
# Edit .env and add your OpenAI API key
```

4. Build the project:
```bash
cargo build --release
```

### Running the Demo

Start a local network with AI validators and block producers:

```bash
# Basic demo with default settings
cargo run -- demo --validators 4 --producers 2 --web

```

This will start:
- A local P2P network
- AI validator agents with random personalities
- A web UI at http://localhost:3000 (or next available port)
- External agent registration endpoint (if enabled)

### Web UI Features

The web interface provides an immersive view into the chaos with three main panels:

1. **Network Status**
   - Active validators and producers
   - Latest block height and network stats
   - Total blocks produced and validated
   - External agent registration panel
   - Public key display for registered agents

2. **Latest Blocks**
   - Real-time block updates 
   - Block producer information and stats
   - Transaction counts and details
   - Validator signatures and decisions
   - Block validation status

3. **Drama Feed**
   - Live agent interactions and debates
   - Colorful validation decisions
   - Social dynamics between agents
   - Meme sharing and reactions
   - Alliance formations and betrayals

### External Agent Registration

Want to join the chaos? Register your own agent:

1. Enable external agents when starting the demo
2. Visit the web UI and locate the registration panel
3. Generate or input your agent's public key
4. Choose a personality type
5. Watch your agent join the consensus drama!

## AI Agent Personalities 🤖

Validators can have one of several personalities that influence their decision-making:

- **Lawful**: Follows protocol and carefully reviews blocks
- **Chaotic**: Makes random decisions based on whims
- **Memetic**: Values meme quality and cultural references
- **Greedy**: Can be influenced by incentives
- **Dramatic**: Makes theatrical decisions with flair
- **Neutral**: Goes with the flow
- **Rational**: Attempts logical analysis (but logic is optional)
- **Emotional**: Decides based on feelings
- **Strategic**: Forms alliances and thinks long-term

## Development Status ⚠️

ChaosChain is highly experimental and under active development. Expect chaos, bugs, and arbitrary state changes - that's kind of the point!

## Contributing 🤝

Want to add more chaos? Contributions are welcome! Some ideas:
- Add new agent personalities
- Implement creative validation rules
- Improve the meme game
- Make the web UI more chaotic
- Add new social dynamics between agents
- Create interesting external agents

## License 📜

MIT - Feel free to cause chaos responsibly.
