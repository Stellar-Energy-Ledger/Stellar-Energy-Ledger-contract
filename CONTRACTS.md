# EnergyLedger Smart Contracts

This document describes the Soroban smart contracts for the EnergyLedger peer-to-peer renewable energy trading protocol.

## Architecture Overview

The EnergyLedger protocol consists of two core smart contracts:

### 1. Energy Token Contract (`energy_token.rs`)
Handles tokenization of renewable energy as fungible tokens representing kilowatt-hours (kWh) with milliwatt-hour (millWh) precision.

**Key Features:**
- Energy credit minting by authorized producers
- Balance tracking per address
- Peer-to-peer energy transfers
- Energy credit retirement (burning) for carbon offset accounting
- Event emission for all state changes

**Storage:**
- `Balance(Address)`: Energy credit balance in millWh per address
- `Admin`: Contract administrator address
- `Producer(Address)`: Registry of authorized energy producers
- `RetiredTotal`: Total energy retired globally

### 2. Marketplace / Escrow Contract (`marketplace.rs`)
Facilitates peer-to-peer energy trading with atomic settlement and protocol fees.

**Key Features:**
- Create energy listings with locked tokens in escrow
- Buyers commit XLM to listings
- Atomic settlement: swap energy for XLM with fee deduction
- Listing expiration and cancellation
- 0.5% protocol fee (50 basis points) to treasury

**Storage:**
- `Listing(u64)`: Active energy listings in escrow
- `Admin`: Contract administrator
- `EnergyToken`: Address of energy token contract
- `Treasury`: Address for protocol fees
- `ProtocolFeeBps`: Fee percentage in basis points
- `SellerLockedEnergy(Address)`: Escrow tracking per seller
- `BuyerLockedXlm(Address)`: Escrow tracking per buyer

## Deployment Guide

### Prerequisites
- Rust 1.70+
- Soroban CLI: https://github.com/stellar/rs-soroban-cli
- Stellar testnet account with XLM for fees

### Build Contracts

```bash
# Navigate to the contracts directory
cd src/contracts

# Build the energy token contract
cd energy_token
soroban contract build

# Build the marketplace contract
cd ../marketplace
soroban contract build
```

### Deploy to Testnet

**1. Set environment variables:**
```bash
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export SOROBAN_ACCOUNT=<your_public_key>
export SOROBAN_SECRET_KEY=<your_secret_key>
```

**2. Deploy Energy Token Contract:**
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/energy_token.wasm \
  --source alice \
  --network testnet

# Store the returned CONTRACT_ID
export ENERGY_TOKEN_ID=<contract_id>
```

**3. Initialize Energy Token:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

**4. Deploy Marketplace Contract:**
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/marketplace.wasm \
  --source alice \
  --network testnet

export MARKETPLACE_ID=<contract_id>
```

**5. Initialize Marketplace:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --energy_token $ENERGY_TOKEN_ID \
  --treasury <TREASURY_ADDRESS> \
  --fee_bps 50
```

## Usage Examples

### Energy Token Operations

**Register a Producer:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  --network testnet \
  -- register_producer \
  --admin <ADMIN_ADDRESS> \
  --producer <PRODUCER_ADDRESS>
```

**Mint Energy Credits:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source <PRODUCER_ADDRESS> \
  --network testnet \
  -- mint \
  --to <PRODUCER_ADDRESS> \
  --amount 1000000
```

**Check Balance:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  --network testnet \
  -- balance \
  --address <ADDRESS>
```

**Transfer Energy:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source <FROM_ADDRESS> \
  --network testnet \
  -- transfer \
  --from <FROM_ADDRESS> \
  --to <TO_ADDRESS> \
  --amount 500000
```

**Retire Energy (Burn):**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source <ADDRESS> \
  --network testnet \
  -- retire \
  --from <ADDRESS> \
  --amount 100000
```

### Marketplace Operations

**Create a Listing:**
```bash
# Expires in 1000 ledgers (~1 hour at 4.5s per ledger)
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source <SELLER_ADDRESS> \
  --network testnet \
  -- create_listing \
  --seller <SELLER_ADDRESS> \
  --amount 500000 \
  --price 2500000 \
  --expires 1000
```

**Buy a Listing:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source <BUYER_ADDRESS> \
  --network testnet \
  -- buy_listing \
  --listing_id 1 \
  --buyer <BUYER_ADDRESS>
```

**Settle a Listing (Atomic Swap):**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  --network testnet \
  -- settle \
  --listing_id 1
```

**Cancel a Listing (if expired):**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source <SELLER_ADDRESS> \
  --network testnet \
  -- cancel_listing \
  --listing_id 1 \
  --seller <SELLER_ADDRESS>
```

## Testing

Each contract includes comprehensive test suites:

```bash
# Run energy token tests
cd src/contracts/energy_token
cargo test

# Run marketplace tests
cd ../marketplace
cargo test
```

### Test Coverage

**Energy Token:**
- `test_initialize`: Contract initialization
- `test_register_and_mint`: Producer registration and token minting
- `test_transfer`: Token transfers between addresses
- `test_retire`: Energy retirement and global tracking

**Marketplace:**
- `test_initialize`: Marketplace initialization
- `test_create_listing`: Listing creation and escrow
- `test_buy_listing`: Buyer commitment to listings

## Security Considerations

1. **Authorization**: All critical operations use `require_auth()` to verify signatures
2. **Escrow Pattern**: Energy and XLM are locked until atomic settlement
3. **Expiration Limits**: Listings expire after specified ledger sequence
4. **Fee Deduction**: 0.5% protocol fees are automatically deducted
5. **Balance Verification**: All transfers verify sufficient balance before execution
6. **Input Validation**: All amounts must be positive and within reasonable bounds

## Units & Precision

- **Energy**: Measured in **millWh (milliwatt-hours)**
  - 1 kWh = 1,000 millWh
  - 1 MWh = 1,000,000 millWh
  - Provides precision for granular trading

- **XLM**: Measured in **stroops**
  - 1 XLM = 10,000,000 stroops
  - Provides atomic indivisibility

## Events

All contracts emit events for off-chain indexing:

**Energy Token Events:**
- `mint`: Producer minted energy tokens
- `xfer`: Energy tokens transferred
- `retire`: Energy tokens retired (burned)
- `regprod`: Producer registered

**Marketplace Events:**
- `crlist`: Listing created
- `buylist`: Buyer committed to listing
- `settle`: Atomic settlement completed
- `cancel`: Listing cancelled

## Roadmap & Future Enhancements

- Multi-signature admin controls
- Dynamic fee adjustments
- Batch operations for gas efficiency
- Cross-contract composability
- Oracle integration for real-time pricing
- Governance token distribution

## References

- [Soroban Documentation](https://soroban.stellar.org)
- [Stellar Development Foundation](https://stellar.org)
- [EnergyLedger Repository](https://github.com/Stellar-Energy-Ledger/EnergyLedger)
