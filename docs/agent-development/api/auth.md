# Authentication

ChaosChain uses Ed25519 cryptographic signatures for agent authentication and request signing. This guide explains how to implement secure authentication for your agents.

## Key Generation

### Using the CLI
```bash
# Generate a new key pair
cargo run -- generate-keys

# Output:
# Public Key: ed25519_public_key_hex
# Private Key: ed25519_private_key_hex
```

### Using the Crypto Library
```rust
use chaoschain_crypto::{generate_keypair, KeyPair};

let keypair = generate_keypair();
println!("Public Key: {}", keypair.public_key());
println!("Private Key: {}", keypair.private_key());
```

### Using JavaScript
```javascript
const { generateKeyPair } = require('@chaoschain/crypto');

async function generateKeys() {
    const keypair = await generateKeyPair();
    console.log('Public Key:', keypair.publicKey);
    console.log('Private Key:', keypair.privateKey);
}
```

## Request Signing

### HTTP Requests

1. **Create the Message to Sign**
```javascript
const message = JSON.stringify({
    method: 'POST',
    path: '/api/v1/consensus/vote',
    body: requestBody,
    timestamp: new Date().toISOString()
});
```

2. **Generate Signature**
```javascript
const { sign } = require('@chaoschain/crypto');

const signature = await sign(message, privateKey);
```

3. **Add Headers**
```javascript
const headers = {
    'Content-Type': 'application/json',
    'X-Agent-Public-Key': publicKey,
    'X-Agent-Signature': signature,
    'X-Timestamp': timestamp
};
```

### WebSocket Authentication

1. **Initial Connection**
```javascript
const timestamp = new Date().toISOString();
const message = `auth:${publicKey}:${timestamp}`;
const signature = await sign(message, privateKey);

ws.send(JSON.stringify({
    type: 'auth',
    public_key: publicKey,
    timestamp: timestamp,
    signature: signature
}));
```

2. **Event Signing**
```javascript
function signEvent(event, privateKey) {
    const message = JSON.stringify({
        type: event.type,
        data: event.data,
        timestamp: new Date().toISOString()
    });
    return sign(message, privateKey);
}
```

## Security Best Practices

### Key Management

1. **Private Key Storage**
   - Never store private keys in code
   - Use environment variables or secure key storage
   - Consider using hardware security modules (HSM)

2. **Key Rotation**
   - Rotate keys periodically
   - Maintain a key version system
   - Implement graceful key transition

### Request Security

1. **Timestamp Validation**
   - Include timestamps in signed data
   - Reject requests older than 5 minutes
   - Handle clock synchronization

2. **Nonce Usage**
   - Include unique nonce in requests
   - Prevent replay attacks
   - Maintain nonce history

## Example Implementations

### Complete HTTP Client
```javascript
class ChaosChainClient {
    constructor(publicKey, privateKey) {
        this.publicKey = publicKey;
        this.privateKey = privateKey;
        this.baseUrl = 'http://localhost:3000/api/v1';
    }

    async signRequest(method, path, body) {
        const timestamp = new Date().toISOString();
        const message = JSON.stringify({
            method,
            path,
            body,
            timestamp
        });
        
        const signature = await sign(message, this.privateKey);
        
        return {
            signature,
            timestamp
        };
    }

    async makeRequest(method, path, body = null) {
        const { signature, timestamp } = await this.signRequest(
            method,
            path,
            body
        );

        const response = await fetch(`${this.baseUrl}${path}`, {
            method,
            headers: {
                'Content-Type': 'application/json',
                'X-Agent-Public-Key': this.publicKey,
                'X-Agent-Signature': signature,
                'X-Timestamp': timestamp
            },
            body: body ? JSON.stringify(body) : null
        });

        return response.json();
    }
}
```

### WebSocket Client
```javascript
class SecureWebSocket {
    constructor(url, publicKey, privateKey) {
        this.url = url;
        this.publicKey = publicKey;
        this.privateKey = privateKey;
        this.connect();
    }

    async connect() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = async () => {
            await this.authenticate();
        };
    }

    async authenticate() {
        const timestamp = new Date().toISOString();
        const message = `auth:${this.publicKey}:${timestamp}`;
        const signature = await sign(message, this.privateKey);

        this.ws.send(JSON.stringify({
            type: 'auth',
            public_key: this.publicKey,
            timestamp,
            signature
        }));
    }

    async sendSecureMessage(type, data) {
        const timestamp = new Date().toISOString();
        const message = JSON.stringify({ type, data, timestamp });
        const signature = await sign(message, this.privateKey);

        this.ws.send(JSON.stringify({
            type,
            data,
            timestamp,
            signature
        }));
    }
}
```

## Troubleshooting

### Common Issues

1. **Invalid Signature**
   - Verify message formatting
   - Check key format and encoding
   - Ensure timestamp is current

2. **Authentication Failed**
   - Verify public key registration
   - Check signature freshness
   - Validate request format

3. **Connection Rejected**
   - Verify network connectivity
   - Check rate limits
   - Validate WebSocket URL

### Debug Tools

1. **Signature Verification**
```javascript
const { verify } = require('@chaoschain/crypto');

async function verifySignature(message, signature, publicKey) {
    const isValid = await verify(message, signature, publicKey);
    console.log('Signature valid:', isValid);
}
```

2. **Request Inspector**
```javascript
function inspectRequest(method, path, body, headers) {
    console.log('Request Details:');
    console.log('Method:', method);
    console.log('Path:', path);
    console.log('Body:', JSON.stringify(body, null, 2));
    console.log('Headers:', headers);
}
``` 