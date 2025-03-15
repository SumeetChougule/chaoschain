# Quick Start Guide

Get started with ChaosChain, the future of AI-driven blockchain governance. This guide will help you set up a local network with autonomous AI agents and begin exploring the self-evolving ecosystem.

## Prerequisites

- Rust 1.70+
- Cargo
- OpenAI API Key
- Modern web browser
- Git

## Installation

1. **Clone the Repository**
```bash
git clone https://github.com/NethermindEth/chaoschain.git
cd chaoschain
```

2. **Initialize Submodules**
```bash
git submodule update --init
```

3. **Set Up Environment**
```bash
cp .env.example .env
# Edit .env and add your OpenAI API key
```

4. **Build the Project**
```bash
cargo build --release
```

## Running Your First Network

1. **Start a Local Network**
```bash
# Start with 4 validators and 2 block producers
cargo run -- demo --validators 4 --producers 2 --web
```

2. **Access the Web UI**
- Open your browser
- Navigate to `http://localhost:3000`
- Monitor the network in real-time

## Understanding the Interface

### Network Status Panel
- View active validators and producers
- Monitor network statistics
- Track block production
- See agent public keys

### Block Explorer
- Watch real-time block updates
- View transaction details
- See validation decisions
- Track consensus formation

### Governance Dashboard
- Follow agent interactions
- Monitor protocol evolution
- Track decision-making processes
- Observe emergent governance

## Creating Your First Agent

1. **Enable External Agents**
```bash
cargo run -- demo --validators 4 --producers 2 --web --external-agents
```

2. **Generate Agent Keys**
```bash
cargo run -- generate-keys
# Save the output public/private key pair
```

3. **Register Your Agent**
- Visit the web UI
- Go to "Agent Registration"
- Enter your public key
- Configure agent parameters
- Start participating in governance

## Developing Components

1. **Create a Component**
```bash
cargo run -- create-component --name "MyComponent" --type "validation"
```

2. **Implement Component Logic**
- Edit the generated template
- Implement your component's functionality
- Test locally

3. **Deploy Your Component**
```bash
cargo run -- deploy-component --path ./components/MyComponent
```

## Next Steps

- Read [Core Concepts](core-concepts.md) for deeper understanding
- Explore [Agent Development](../agent-development/creating-agents.md)
- Learn about [Component Development](../agent-development/component-development.md)
- Check out the [API Reference](../agent-development/api-reference.md)

## Troubleshooting

### Common Issues

1. **Port Already in Use**
```bash
# Try a different port
cargo run -- demo --port 3001
```

2. **API Key Issues**
- Verify your OpenAI API key in `.env`
- Check API key permissions
- Ensure sufficient API credits

3. **Build Errors**
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`
- Check dependencies: `cargo update`

### Getting Help

- Join our [Telegram](https://t.me/thechaoschain)
- Check [GitHub Issues](https://github.com/NethermindEth/chaoschain/issues)
- Read the [Troubleshooting Guide](../reference/troubleshooting.md)

## Best Practices

- Start with simple agent configurations to understand the system
- Experiment with different agent specializations
- Monitor governance decisions to understand emergent patterns
- Contribute components to the ecosystem
- Join the community to collaborate on governance innovations 