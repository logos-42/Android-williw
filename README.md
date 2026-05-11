# Williw - AI Compute Power API

A mobile-first Rust project providing AI compute power APIs with multi-payment support.

## Features

- **Compute Power API**: Access various AI models (LLM, Image, Audio, Video, Multimodal)
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

## Project Structure

```
williw/
├── Cargo.toml              # Workspace root
├── api/                    # Backend API (Axum)
│   └── src/
│       ├── main.rs
│       ├── routes/         # API routes
│       ├── models/         # Data models
│       ├── services/       # Business logic
│       ├── auth/           # JWT + Wallet auth
│       ├── payments/       # WeChat/Alipay/Crypto
│       └── db/             # Database layer
├── frontend/               # Frontend (Leptos)
│   └── src/
│       ├── pages/          # Login, Models, Payment, Orders, Profile
│       ├── components/      # Reusable UI components
│       └── api/            # API client
└── shared/                  # Shared types
```

## API Endpoints

### Auth
- `POST /api/auth/wallet/login` - Login with crypto wallet signature
- `POST /api/auth/wallet/verify` - Verify wallet ownership
- `GET /api/auth/profile` - Get user profile

### Compute Power (算力)
- `GET /api/compute/models` - List available AI models
- `GET /api/compute/models/:id` - Get model details
- `POST /api/compute/request` - Request compute power
- `GET /api/compute/status/:id` - Check job status

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

## Pages

1. **Login** - Enter crypto wallet address to login
2. **Models** - Browse and filter AI models
3. **Model Detail** - View specs and request compute
4. **Payment** - Pay with WeChat/Alipay/Crypto
5. **Orders** - View order history
6. **Profile** - Wallet info and settings

## Mobile-First Design

- Responsive grid: 1 col mobile, 2-3 col tablet, 4 col desktop
- Touch-friendly: 44px min tap targets
- Bottom navigation bar for mobile
