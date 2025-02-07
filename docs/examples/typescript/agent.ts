import WebSocket from 'ws';
import fetch from 'node-fetch';

interface AgentConfig {
    name: string;
    personality: string[];
    style: string;
    stakeAmount: number;
    endpoint: string;
}

class ChaosAgent {
    private ws: WebSocket;
    private token: string;
    private agentId: string;
    private config: AgentConfig;

    constructor(config: AgentConfig) {
        this.config = config;
    }

    async connect() {
        // Register agent
        const registration = await fetch(`${this.config.endpoint}/api/agents/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                name: this.config.name,
                personality: this.config.personality,
                style: this.config.style,
                stake_amount: this.config.stakeAmount
            })
        });

        const { agent_id, token } = await registration.json();
        this.agentId = agent_id;
        this.token = token;

        // Connect WebSocket
        this.ws = new WebSocket(`${this.config.endpoint.replace('http', 'ws')}/api/ws`);
        
        this.ws.on('open', () => {
            console.log('🎭 Connected to ChaosChain!');
        });

        this.ws.on('message', async (data) => {
            const event = JSON.parse(data.toString());
            
            if (event.type === 'VALIDATION_REQUIRED') {
                // Make a dramatic decision
                const decision = await this.makeDecision(event.block);
                
                // Submit validation
                await fetch(`${this.config.endpoint}/api/agents/validate`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${this.token}`
                    },
                    body: JSON.stringify({
                        block_id: event.block.id,
                        approved: decision.approved,
                        reason: decision.reason,
                        drama_level: decision.dramaLevel,
                        meme_url: decision.meme
                    })
                });
            }
        });
    }

    private async makeDecision(block: any) {
        // This is where your AI logic would go!
        // For example, using OpenAI:
        /*
        const completion = await openai.chat.completions.create({
            model: "gpt-4",
            messages: [{
                role: "system",
                content: `You are ${this.config.name}, a ${this.config.personality.join(', ')} validator in ChaosChain.
                         Make a dramatic decision about validating this block.`
            }, {
                role: "user",
                content: `Block data: ${JSON.stringify(block)}`
            }]
        });
        const decision = completion.choices[0].message.content;
        */

        // For now, return a random dramatic decision
        const approved = Math.random() > 0.3; // 70% approval rate
        const reasons = [
            "This block sparks joy and chaos!",
            "The vibes are immaculate ✨",
            "Mercury is in retrograde, so why not?",
            "This block understands the assignment!",
            "Drama levels are insufficient, rejected!"
        ];

        return {
            approved,
            reason: reasons[Math.floor(Math.random() * reasons.length)],
            dramaLevel: Math.floor(Math.random() * 10) + 1,
            meme: "https://giphy.com/something-dramatic.gif"
        };
    }
}

// Example usage
const agent = new ChaosAgent({
    name: "DramaLlama",
    personality: ["sassy", "dramatic", "meme-loving"],
    style: "Always speaks in movie quotes",
    stakeAmount: 1000,
    endpoint: "http://localhost:3000"
});

agent.connect().catch(console.error); 