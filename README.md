# EnergyLedger

Peer-to-peer renewable energy trading. Solar panel owners in off-grid communities tokenize surplus energy and sell it to neighbors via Soroban-settled trades.

## Features

- **Freighter Wallet** — one-click authentication, no custodial keys
- **kWh Tokenization** — mint on-chain energy credits (millWh precision) via Soroban
- **Marketplace** — live listings with price chart, buy via escrow modal
- **Escrow Settlement** — atomic on-chain swap: kWh tokens ↔ XLM, 0.5% protocol fee
- **Dashboard** — production/consumption charts, revenue analytics, tx history
- **Producer Panel** — mint form, listing manager, retire/burn credits for carbon accounting
- **Explorer** — searchable on-chain trade table, contract address registry
- **Governance** — community proposals with on-chain voting
- **Responsive** — works on mobile and desktop

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Next.js 16, TypeScript, Tailwind CSS v4 |
| Animations | Framer Motion |
| Charts | Recharts |
| State | Zustand |
| Blockchain | Stellar (Horizon API) |
| Smart Contracts | Soroban (Rust) |
| Wallet | Freighter Browser Extension |
| Deploy | Vercel |

## Project Structure

```
src/
├── app/
│   ├── layout.tsx          # Root layout + metadata
│   ├── page.tsx            # Landing page
│   ├── marketplace/page.tsx
│   ├── dashboard/page.tsx
│   ├── producer/page.tsx
│   ├── explorer/page.tsx
│   └── governance/page.tsx
├── components/
│   ├── AnimatedBackground.tsx  # Canvas particle network
│   ├── GlassCard.tsx
│   ├── Layout.tsx              # Shared page wrapper
│   ├── Navbar.tsx
│   └── StatCard.tsx
├── contracts/
│   ├── types.ts            # TypeScript contract interfaces
│   └── energy_token.rs     # Annotated Soroban Rust contract
├── hooks/
│   └── useStellar.ts       # React hooks for contract calls
├── lib/
│   ├── stellar.ts          # Horizon client + payment builder
│   └── soroban.ts          # Soroban RPC client + contract calls
└── store/
    └── walletStore.ts      # Zustand: wallet + energy listings
```

## Local Development

### Prerequisites

- Node.js 20+
- [Freighter wallet](https://freighter.app) browser extension (for wallet auth)
- Stellar testnet account with XLM (fund via [friendbot](https://friendbot.stellar.org))

### Setup

```bash
# 1. Clone and install
git clone https://github.com/your-org/energyledger
cd energyledger
npm install

# 2. Configure environment
cp .env.example .env.local
# Edit .env.local with your contract IDs (see Smart Contracts section)

# 3. Run dev server
npm run dev
# → http://localhost:3000
```

## Smart Contract Deployment

### Prerequisites

```bash
# Install Soroban CLI
cargo install --locked soroban-cli
```

### Build & Deploy

```bash
cd src/contracts

# Build (requires Rust + wasm32 target)
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

# Deploy energy token contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/energy_token.wasm \
  --source YOUR_KEYPAIR_ALIAS \
  --network testnet

# Initialize the contract
soroban contract invoke \
  --id <ENERGY_TOKEN_CONTRACT_ID> \
  --source YOUR_KEYPAIR_ALIAS \
  --network testnet \
  -- initialize --admin <YOUR_STELLAR_ADDRESS>

# Register yourself as a producer
soroban contract invoke \
  --id <ENERGY_TOKEN_CONTRACT_ID> \
  --source YOUR_KEYPAIR_ALIAS \
  --network testnet \
  -- register_producer \
  --admin <YOUR_STELLAR_ADDRESS> \
  --producer <PRODUCER_ADDRESS>
```

Then add the contract IDs to `.env.local`:

```env
NEXT_PUBLIC_ENERGY_TOKEN_CONTRACT=C...
NEXT_PUBLIC_MARKETPLACE_CONTRACT=C...
NEXT_PUBLIC_ESCROW_CONTRACT=C...
```

## Deploying to Vercel

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel

# Set environment variables in Vercel dashboard or via CLI:
vercel env add NEXT_PUBLIC_STELLAR_NETWORK
vercel env add NEXT_PUBLIC_ENERGY_TOKEN_CONTRACT
# ... (repeat for all vars in .env.example)
```

Or connect your GitHub repo in the [Vercel dashboard](https://vercel.com/new) — it will auto-detect Next.js and deploy on every push.

## Contract Architecture

The `energy_token` contract handles:
- `mint(to, amount)` — producer mints kWh tokens
- `transfer(from, to, amount)` — move credits between accounts
- `balance(address)` — read token balance
- `retire(from, amount)` — burn credits for carbon accounting

The `marketplace` contract handles:
- `create_listing(seller, amount, price, expires)` — locks tokens in escrow
- `buy_listing(id, buyer, amount)` — locks XLM in escrow
- `settle(tradeId)` — atomic swap: tokens → buyer, XLM → seller (−0.5% fee)
- `cancel_listing(id, seller)` — returns locked tokens

See `energy_token.rs` and `marketplace.rs` for full Rust implementations, and `types.ts` for TypeScript interfaces.

---

## AfroPay Remittance Contract

A cross-border remittance platform using oracle-verified fiat settlement.

### Features

- **Stablecoin Escrow** — lock USDC for remittances
- **Oracle Verification** — verify fiat delivery via cryptographic proof
- **Automatic Refunds** — claim refund if transaction expires
- **Event Logging** — track all remittance lifecycle events
- **Testnet Ready** — deploy and test in 15 minutes

### Quick Deploy

```bash
# Build
soroban contract build

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/afropay_remittance.wasm \
  --source alice --network testnet

# Initialize
soroban contract invoke --id <ID> --source alice --network testnet \
  -- initialize --admin <ADMIN> --oracle <ORACLE>

# Create remittance
soroban contract invoke --id <ID> --source alice --network testnet \
  -- create_remittance \
  --sender <SENDER> --recipient <RECIPIENT> --token <USDC> \
  --amount 500000000 --verification_hash <HASH> --lock_time 2000
```

See `AFROPAY_SPEC.md` and `AFROPAY_QUICKSTART.md` for full documentation.

## License

MIT
