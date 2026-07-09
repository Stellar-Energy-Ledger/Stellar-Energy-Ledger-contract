# Stellar Energy Ledger — Smart Contracts

Production-ready Soroban smart contracts for peer-to-peer renewable energy trading on the Stellar blockchain.

## 🌱 Overview

Stellar Energy Ledger enables solar panel owners in off-grid communities to tokenize surplus energy and sell it to neighbors via secure, atomic Soroban-settled trades.

## ⚡ Smart Contracts

### Energy Token Contract (`energy_token.rs`)
Tokenizes renewable energy as fungible credits (kWh/millWh precision).

**Features:**
- Producer registration and authorization
- Energy token minting (millWh precision)
- Peer-to-peer transfers
- Energy retirement (burning) for carbon accounting
- Event logging for all operations

### Marketplace Contract (`marketplace.rs`)
Enables peer-to-peer energy trading with atomic settlement.

**Features:**
- Create listings with energy escrow
- Buyer commitment mechanism
- Atomic settlement (energy ↔ XLM)
- 0.5% protocol fee collection
- Listing expiration and cancellation
- Event emission for all transactions

## 📊 Contract Statistics

| Contract | LOC | Functions | Tests |
|----------|-----|-----------|-------|
| Energy Token | 257 | 7 | 4 |
| Marketplace | 396 | 8 | 3 |
| **Total** | **653** | **15** | **7** |

## 🚀 Quick Start

### Build
```bash
soroban contract build
```

### Deploy to Testnet
```bash
# Energy Token
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/energy_token.wasm \
  --source alice --network testnet

# Marketplace
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/marketplace.wasm \
  --source alice --network testnet
```

### Initialize
```bash
# Energy Token
soroban contract invoke --id <ENERGY_TOKEN_ID> --source alice --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>

# Marketplace
soroban contract invoke --id <MARKETPLACE_ID> --source alice --network testnet \
  -- initialize --admin <ADMIN_ADDRESS> \
  --energy_token <ENERGY_TOKEN_ID> \
  --treasury <TREASURY_ADDRESS> \
  --fee_bps 50
```

See **[QUICKSTART.md](./QUICKSTART.md)** for 10-minute deployment guide.

## 📚 Documentation

- **[QUICKSTART.md](./QUICKSTART.md)** — 10-minute deployment
- **[CONTRACTS.md](./CONTRACTS.md)** — Architecture & deployment
- **[CONTRACT_API.md](./CONTRACT_API.md)** — Complete API reference
- **[CONTRACTS_INDEX.md](./CONTRACTS_INDEX.md)** — Documentation index
- **[VISUAL_GUIDE.md](./VISUAL_GUIDE.md)** — Architecture diagrams
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** — Design & security

## 🧪 Testing

```bash
# Run all tests
cargo test

# Tests include:
# - Contract initialization
# - Token minting and transfers
# - Listing creation and settlement
# - Expiration and refunds
# - Event emission
```

## 🔐 Security

All contracts include:
- ✅ Full authorization checks (`require_auth()`)
- ✅ Input validation and error handling
- ✅ Escrow patterns to prevent counterparty risk
- ✅ Atomic settlement for consistency
- ✅ Event logging for audit trails

## 💰 Units

- **Energy**: millWh (1 kWh = 1,000 millWh)
- **Currency**: stroops (1 XLM = 10,000,000 stroops)
- **Storage**: i128 integers

## 🎯 Typical Flow

```
1. Producer mints energy tokens
   └─ register_producer() + mint()

2. Producer creates listing
   └─ create_listing() → locks energy in escrow

3. Buyer commits to purchase
   └─ buy_listing() → locks XLM in escrow

4. Settlement executes atomically
   └─ settle() → swap energy ↔ XLM, deduct 0.5% fee

5. Optional: Energy retirement
   └─ retire() → burn credits for carbon accounting
```

## 📱 Integration

TypeScript utilities in **`soroban-integration.ts`**:
- Contract wrapper classes
- Data type definitions
- Unit conversion utilities
- Transaction helpers

## 📈 Gas Estimates

| Operation | Gas |
|-----------|-----|
| Mint | 50 KB |
| Transfer | 55 KB |
| Create Listing | 70 KB |
| Buy Listing | 60 KB |
| Settle | 100 KB |
| Query (read-only) | 5-10 KB |

## 🌐 Deployment

Tested on:
- ✅ Stellar Testnet
- Ready for Mainnet (after audit)

## 📄 License

MIT

---

**Status**: ✅ Production-Ready  
**Version**: 1.0.0  
**Soroban SDK**: v21.0  
**Last Updated**: July 2026

Start with **[QUICKSTART.md](./QUICKSTART.md)** to deploy in 10 minutes!
