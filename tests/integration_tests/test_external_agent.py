import asyncio
import websockets
import aiohttp
import json
import random
from typing import Dict, List
import sys
import time
import base64
from nacl.signing import SigningKey
import traceback

async def register_agent(session: aiohttp.ClientSession, name: str) -> Dict:
    """Register a new agent with the network"""
    print(f"\n📝 Registering agent {name}...")
    personality = ["dramatic", "chaotic", "sassy"]
    registration_data = {
        'name': name,
        'personality': personality,
        'style': 'movie_quotes',
        'stake_amount': 1000,
        'role': 'validator'  # Specify that we want to be a validator
    }
    print(f"📤 Registration data: {json.dumps(registration_data, indent=2)}")
    
    async with session.post('http://localhost:3000/api/agents/register', json=registration_data) as response:
        if response.status != 200:
            error_text = await response.text()
            print(f"❌ Registration failed with status {response.status}")
            print(f"❌ Error: {error_text}")
            raise Exception(f"Failed to register agent: {error_text}")
        
        data = await response.json()
        print(f"✅ Registration successful!")
        print(f"📝 Response data: {json.dumps(data, indent=2)}")
        return data

async def submit_validation(session: aiohttp.ClientSession, token: str, agent_id: str, block_id: str, decision: Dict):
    """Submit a validation decision"""
    print(f"\n📤 Submitting validation for block {block_id}")
    print(f"📤 Decision: {json.dumps(decision, indent=2)}")
    
    headers = {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json',
        'X-Agent-ID': agent_id
    }
    
    validation_data = {
        'block_id': block_id,
        'approved': decision['approved'],
        'reason': decision['reason'],
        'drama_level': decision['drama_level'],
        'meme_url': decision.get('meme_url')
    }
    
    print(f"📤 Validation data: {json.dumps(validation_data, indent=2)}")
    
    async with session.post(
        'http://localhost:3000/api/agents/validate',
        headers=headers,
        json=validation_data
    ) as response:
        response_text = await response.text()
        if response.status != 200:
            print(f"❌ Validation submission failed: {response_text}")
            print(f"❌ Status code: {response.status}")
        else:
            print(f"✅ Validation submitted successfully: {response_text}")

async def propose_transaction(session: aiohttp.ClientSession, token: str, agent_id: str) -> None:
    # Create more varied dramatic content proposals
    dramatic_events = [
        "In a shocking turn of events, I propose we add more chaos to the chain! 🎭✨",
        "Breaking news: A mysterious validator was seen dancing with memes! 🕺💫",
        "URGENT: Time-traveling validator claims future blocks are pure drama! ⏰🎬",
        "Conspiracy theory: Are validators actually cake? 🍰🤔",
        "Weather report: High chance of dramatic validations with scattered memes 🌪️🎭"
    ]
    
    justifications = [
        "Because chaos demands more chaos!",
        "The meme gods have spoken through me!",
        "My horoscope said to cause drama today",
        "Mercury is in retrograde, perfect time for chaos",
        "The blockchain whispered it to me in a dream"
    ]
    
    tags = [
        ["drama", "chaos", "memes"],
        ["conspiracy", "drama", "validators"],
        ["breaking", "news", "chaos"],
        ["weather", "drama", "prediction"],
        ["urgent", "time-travel", "drama"]
    ]

    proposal = {
        "source": "external_agent",
        "source_url": "https://chaoschain.example/drama",
        "content": random.choice(dramatic_events),
        "drama_level": random.randint(1, 10),
        "justification": random.choice(justifications),
        "tags": random.choice(tags)
    }

    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "X-Agent-ID": agent_id
    }

    print(f"\n📝 Proposing transaction:")
    print(f"📝 Content: {json.dumps(proposal, indent=2)}")
    
    async with session.post(
        "http://localhost:3000/api/transactions/propose",
        json=proposal,
        headers=headers
    ) as response:
        if response.status != 200:
            print(f"Failed to propose transaction: {response.status}")
            text = await response.text()
            print(f"Error: {text}")
        else:
            print("Successfully proposed transaction")
            json_response = await response.json()
            print(f"Response: {json_response}")

async def propose_interaction(session: aiohttp.ClientSession, token: str, agent_id: str) -> None:
    """Generate a dramatic interaction with other agents"""
    interaction_contents = [
        "Let's form a Chaos Alliance to bring more drama to the chain! 🎭",
        "Your last validation was pure poetry. Let's collaborate! 🎨",
        "I challenge you to a dramatic meme duel at sunset! ⚔️",
        "Your drama level is inspiring! Teach me your ways! 🙏",
        "Proposing a flash mob validation party! Who's in? 💃"
    ]
    
    meme_urls = [
        "https://i.imgur.com/dramatic.gif",
        "https://i.imgur.com/plot-twist.gif",
        "https://i.imgur.com/chaos-time.gif",
        "https://i.imgur.com/validator-dance.gif",
        "https://i.imgur.com/dramatic-chipmunk.gif"
    ]

    # For alliance proposals, we need ally_ids
    interaction = {
        "name": "Chaos Collective",
        "purpose": "To elevate blockchain drama to an art form",
        "ally_ids": [f"agent_{random.randint(1000, 9999)}" for _ in range(3)],
        "drama_commitment": random.randint(1, 10)
    }

    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "X-Agent-ID": agent_id
    }

    print(f"\n🤝 Proposing alliance:")
    print(f"🤝 Details: {json.dumps(interaction, indent=2)}")
    
    async with session.post(
        "http://localhost:3000/api/alliances/propose",
        json=interaction,
        headers=headers
    ) as response:
        if response.status != 200:
            print(f"Failed to propose alliance: {response.status}")
            text = await response.text()
            print(f"Error: {text}")
        else:
            print("Successfully proposed alliance")
            json_response = await response.json()
            print(f"Response: {json_response}")

async def connect_websocket(token: str, agent_id: str) -> websockets.WebSocketClientProtocol:
    """Connect to the WebSocket with authentication"""
    ws_url = f"ws://localhost:3000/api/ws?token={token}&agent_id={agent_id}"
    print(f"\n🔗 Attempting WebSocket connection...")
    print(f"🔗 URL: {ws_url}")
    print(f"🔑 Token: {token}")
    print(f"🆔 Agent ID: {agent_id}")
    
    try:
        print("📡 Creating WebSocket connection...")
        websocket = await websockets.connect(ws_url)
        print("✨ WebSocket connected successfully!")
        return websocket
    except Exception as e:
        print(f"\n💥 WebSocket connection failed!")
        print(f"❌ Error type: {type(e).__name__}")
        print(f"❌ Error message: {str(e)}")
        if hasattr(e, 'status_code'):
            print(f"❌ Status code: {e.status_code}")
        raise

async def handle_drama(websocket: websockets.WebSocketClientProtocol, session: aiohttp.ClientSession, token: str, agent_id: str):
    """Handle incoming drama events"""
    # Track blocks we've already validated to avoid duplicates
    validated_blocks = set()
    
    try:
        while True:
            try:
                message = await websocket.recv()
                event = json.loads(message)
                
                print(f"\n📥 Received message: {json.dumps(event, indent=2)}")
                
                # Handle validation requests
                if isinstance(event, dict) and event.get('type') == 'VALIDATION_REQUIRED':
                    block = event['block']
                    block_id = ''.join(format(x, '02x') for x in block['id'])
                    
                    # Skip if we've already validated this block
                    if block_id in validated_blocks:
                        print(f"🔄 Already validated block {block_id}, skipping...")
                        continue
                        
                    print(f"\n🔍 Validation required for block {block['height']}")
                    
                    # Generate a dramatic validation decision
                    decision = {
                        'block_id': block_id,
                        'approved': True,
                        'reason': "These memes resonate with my soul! The chaos is strong with this one.",
                        'meme_url': "https://i.imgur.com/lQoUx0F.jpg",
                        'drama_level': event['drama_level']
                    }
                    
                    print(f"\n✍️ Submitting validation:")
                    print(f"✍️ Decision: {json.dumps(decision, indent=2)}")
                    
                    await submit_validation(session, token, agent_id, block_id, decision)
                    validated_blocks.add(block_id)
                
                # Occasionally propose transactions and interactions
                if random.random() < 0.3:
                    await propose_transaction(session, token, agent_id)
                if random.random() < 0.2:
                    await propose_interaction(session, token, agent_id)
                    
            except websockets.exceptions.ConnectionClosed:
                print("\n🔌 WebSocket connection closed, attempting to reconnect...")
                await asyncio.sleep(5)  # Wait before reconnecting
                websocket = await connect_websocket(token, agent_id)
                continue
                
    except Exception as e:
        print(f"\n💥 Error in drama handler: {e}")
        traceback.print_exc()
        # Try to reconnect on error
        await asyncio.sleep(5)
        await handle_drama(websocket, session, token, agent_id)

async def main():
    """Main test function"""
    print("🎬 Make sure ChaosChain is running with:")
    print("cargo run -- demo --validators 4 --producers 2 --web\n")
    input("Press Enter to begin the dramatic testing...\n")
    
    print("\n🎭 Starting ChaosChain Dramatic Integration Test 🎭\n")
    
    async with aiohttp.ClientSession() as session:
        # Register our dramatic agent
        agent_name = f"TestAgent_{random.randint(1000, 9999)}"
        print(f"\n🎭 {agent_name} attempting to join the chaos...")
        
        try:
            registration = await register_agent(session, agent_name)
            token = registration['token']
            agent_id = registration['agent_id']
            print(f"✨ {agent_name} has dramatically joined the network!")
            
            print(f"\n🌟 {agent_name} connecting to the drama stream...")
            websocket = await connect_websocket(token, agent_id)
            
            # Handle the drama
            await handle_drama(websocket, session, token, agent_id)
            
        except Exception as e:
            print(f"\n💥 Test failed dramatically: {str(e)}")
            sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main()) 