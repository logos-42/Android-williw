# Williw - AI Compute Power API

A mobile-first Rust project providing AI compute power APIs with multi-payment support and P2P model serving.

## Features

- **Local Model Downloads**: Download AI models (LF2.5, Gamma4, Phi-3.5, Qwen, etc.) to your phone
- **P2P Tunnel**: Share your models with anyone over the internet via peer-to-peer connection
- **Compute Power API**: Access cloud AI models (LLM, Image, Audio, Video, Multimodal)
- **Model Filtering**: Filter by category, provider, power, and price
- **Multi-Payment**: WeChat Pay, Alipay, USDT, ETH, BTC
- **Crypto Wallet Login**: Login with Ethereum wallet signature

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Axum (async Rust) |
| Frontend | Leptos (Rust WASM) |
| Database | SQLite (dev) / PostgreSQL (prod) with SQLx |
| Auth | JWT + Crypto Wallet Signature |
| P2P | STUN + Relay fallback for NAT traversal |

## Project Structure

```
williw/
├── Cargo.toml              # Workspace root
├── api/                    # Backend API (Axum)
│   └── src/
│       ├── main.rs
│       ├── routes/         # API routes (auth, compute, payment, local, p2p)
│       ├── models/         # Request/response models
│       ├── services/       # Business logic
│       ├── auth/           # JWT + Wallet auth
│       ├── payments/       # WeChat/Alipay/Crypto
│       ├── p2p/            # P2P tunnel service
│       └── db/             # Database layer
├── frontend/               # Frontend (Leptos)
│   └── src/
│       ├── pages/          # Home, Login, Models, LocalModels, ApiServer, Payment, Orders, Profile
│       ├── components/     # Reusable UI components
│       └── api/            # API client
└── shared/                 # Shared types
```

## API Endpoints

### Auth
- `POST /api/auth/wallet/login` - Login with crypto wallet signature
- `POST /api/auth/wallet/verify` - Verify wallet ownership
- `GET /api/auth/profile` - Get user profile

### Compute Power (Cloud)
- `GET /api/compute/models` - List available AI models
- `GET /api/compute/models/:id` - Get model details
- `POST /api/compute/request` - Request compute power
- `GET /api/compute/status/:id` - Check job status

### Local Models
- `GET /api/local/models` - List downloadable models
- `POST /api/local/models/download` - Download a model
- `DELETE /api/local/models/:id` - Delete downloaded model
- `POST /api/local/inference` - Run inference
- `GET /api/local/device-info` - Get device storage/memory info

### P2P Tunnel (Internet Access)
- `POST /api/p2p/online` - Go online with P2P
- `POST /api/p2p/offline` - Go offline
- `GET /api/p2p/status` - Get P2P status
- `GET /api/p2p/connection-info` - Get your connection code
- `POST /api/p2p/connect/:peer_id` - Connect to another peer
- `POST /api/p2p/share` - Share your connection
- `GET /api/p2p/config` - Get P2P configuration
- `POST /api/p2p/config` - Update P2P configuration

### Payments
- `POST /api/payment/create` - Create order
- `POST /api/payment/initiate` - Initiate payment
- `GET /api/payment/status/:id` - Payment status
- `GET /api/payment/orders` - User orders

## Environment Variables

```bash
# Database
DATABASE_URL=sqlite:williw.db?mode=rwc

# Crypto Wallet Addresses
USDT_ADDRESS=your_trc20_usdt_address
ETH_ADDRESS=your_eth_address
BTC_ADDRESS=your_btc_address

# P2P Relay Server (optional)
WILLIW_RELAY_URL=wss://relay.williw.ai
```

## Running

### Backend
```bash
cd williw
cargo run -p williw-api
```

### Frontend
```bash
cd williw
cargo build -p williw-frontend
# Serve the WASM output
```

## Available Local Models

| Model | Size | Memory Required |
|-------|------|-----------------|
| LF2.5 7B | 4.2 GB | 6 GB |
| LF2.5 14B | 8.5 GB | 10 GB |
| Gamma 4B | 2.5 GB | 4 GB |
| Gamma 7B | 4.3 GB | 6 GB |
| Phi-3.5 Mini | 2.3 GB | 4 GB |
| Qwen 2.5 7B | 4.4 GB | 6 GB |
| Yi 6B | 3.8 GB | 6 GB |
| DeepSeek 7B | 4.1 GB | 6 GB |
| Llama 3.2 3B | 1.9 GB | 4 GB |
| Mistral 7B | 4.3 GB | 6 GB |
| Gemma 2B | 1.4 GB | 3 GB |

## Pages

1. **Home** - Dashboard with quick access to all features
2. **Login** - Enter crypto wallet address to login
3. **Local Models** - Download and manage AI models on your phone
4. **API Server** - Control local + P2P server, share with others
5. **Models** - Browse and filter cloud AI models
6. **Model Detail** - View specs and request compute
7. **Payment** - Pay with WeChat/Alipay/Crypto
8. **Orders** - View order history
9. **Profile** - Wallet info and settings

## P2P Usage

### Sharing Models Over Internet

1. Download models to your phone (Local Models page)
2. Go to API Server page
3. Click "Go Online" for P2P
4. Share your connection code with friends
5. They can now call your AI models via the P2P endpoint

### Connecting to Remote Models

1. Get a connection code from a friend
2. Go to API Server page
3. Make sure P2P is "Online"
4. Enter their connection code and click "Connect"
5. You now have access to their models!

## Mobile-First Design

- Responsive grid: 1 col mobile, 2-3 col tablet, 4 col desktop
- Touch-friendly: 44px min tap targets
- Bottom navigation bar for mobile
- P2P status indicator always visible
