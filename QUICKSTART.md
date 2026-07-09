# EnergyLedger Smart Contracts — Quick Start

Get the EnergyLedger contracts deployed and running on Stellar Testnet in 10 minutes.

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Soroban CLI: `cargo install soroban-cli`
- Stellar account with testnet XLM (get free from [friendbot.stellar.org](https://friendbot.stellar.org))

## 1. Build the Contracts

```bash
# Build Energy Token Contract
cd src/contracts/energy_token
soroban contract build
cd ..

# Build Marketplace Contract
cd marketplace
soroban contract build
cd ../../..
```

Output: `.wasm` files in `target/wasm32-unknown-unknown/release/`

## 2. Set Environment Variables

```bash
# Testnet RPC
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Your account (created on Testnet)
export SOROBAN_ACCOUNT="GXXXXXX..." # Your public key
export SOROBAN_SECRET_KEY="SXXXXXX..." # Your secret key
```

## 3. Deploy Energy Token Contract

```bash
# Deploy
ENERGY_TOKEN=$(soroban contract deploy \
  --wasm src/contracts/target/wasm32-unknown-unknown/release/energy_token.wasm \
  --source alice \
  --network testnet | tail -1)

echo "Energy Token ID: $ENERGY_TOKEN"

# Initialize
soroban contract invoke \
  --id $ENERGY_TOKEN \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address alice)
```

## 4. Deploy Marketplace Contract

```bash
# Deploy
MARKETPLACE=$(soroban contract deploy \
  --wasm src/contracts/target/wasm32-unknown-unknown/release/marketplace.wasm \
  --source alice \
  --network testnet | tail -1)

echo "Marketplace ID: $MARKETPLACE"

# Initialize (fee = 50 basis points = 0.5%)
soroban contract invoke \
  --id $MARKETPLACE \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address alice) \
  --energy_token $ENERGY_TOKEN \
  --treasury $(soroban keys address alice) \
  --fee_bps 50
```

## 5. Test the Flow

### Register a Producer

```bash
ADMIN=$(soroban keys address alice)
PRODUCER=$(soroban keys address bob)

soroban contract invoke \
  --id $ENERGY_TOKEN \
  --source alice \
  --network testnet \
  -- register_producer \
  --admin $ADMIN \
  --producer $PRODUCER
```

### Producer Mints Energy

```bash
soroban contract invoke \
  --id $ENERGY_TOKEN \
  --source bob \
  --network testnet \
  -- mint \
  --to $PRODUCER \
  --amount 1000000
```

### Check Balance

```bash
soroban contract invoke \
  --id $ENERGY_TOKEN \
  --source alice \
  --network testnet \
  -- balance \
  --address $PRODUCER
```

Expected output: `1000000` (millWh)

### Create a Listing

```bash
FUTURE_EXPIRE=$(($(soroban network ls testnet | grep 'Latest Ledger' | awk '{print $3}') + 1000))

LISTING_ID=$(soroban contract invoke \
  --id $MARKETPLACE \
  --source bob \
  --network testnet \
  -- create_listing \
  --seller $PRODUCER \
  --amount 500000 \
  --price 2500000 \
  --expires $FUTURE_EXPIRE | tail -1)

echo "Listing ID: $LISTING_ID"
```

### Buy the Listing

```bash
BUYER=$(soroban keys address charlie)

soroban contract invoke \
  --id $MARKETPLACE \
  --source charlie \
  --network testnet \
  -- buy_listing \
  --listing_id $LISTING_ID \
  --buyer $BUYER
```

### Settle the Trade

```bash
soroban contract invoke \
  --id $MARKETPLACE \
  --source alice \
  --network testnet \
  -- settle \
  --listing_id $LISTING_ID
```

## 6. Verify Results

```bash
# Check buyer received energy
soroban contract invoke \
  --id $ENERGY_TOKEN \
  --source alice \
  --network testnet \
  -- balance \
  --address $BUYER

# Expected: ~500000 (some gas cost deducted)
```

## 7. Store Contract IDs

Save these for later use:

```bash
cat > .env.contracts <<EOF
NEXT_PUBLIC_ENERGY_TOKEN_ID=$ENERGY_TOKEN
NEXT_PUBLIC_MARKETPLACE_ID=$MARKETPLACE
NEXT_PUBLIC_SOROBAN_RPC_URL=$SOROBAN_RPC_URL
EOF
```

## Troubleshooting

### "Contract not found"
- Check contract ID is correct
- Verify ledger sequence (listable on testnet explorer)

### "Insufficient balance"
- Ensure energy token owner has sufficient funds
- For marketplace: ensure buyer has enough XLM

### "Authorization failed"
- Verify source account matches `require_auth()` requirements
- Check secret key is set correctly

### "Expired listing"
- Increase `--expires` value (current ledger + larger offset)
- On testnet, ~4-5 seconds per ledger

## Next Steps

1. **Frontend Integration**: Update `src/lib/soroban-integration.ts` with contract IDs
2. **Dashboard**: Build UI on `src/app/dashboard/page.tsx`
3. **Events**: Set up event subscriptions for real-time updates
4. **Testing**: Run `cargo test` for local contract testing

## Useful Commands

```bash
# View listing
soroban contract invoke \
  --id $MARKETPLACE \
  --source alice \
  --network testnet \
  -- get_listing \
  --listing_id 1

# Check locked energy
soroban contract invoke \
  --id $MARKETPLACE \
  --source alice \
  --network testnet \
  -- seller_locked_energy \
  --seller $PRODUCER

# Check protocol fee
soroban contract invoke \
  --id $MARKETPLACE \
  --source alice \
  --network testnet \
  -- protocol_fee_bps
```

## Reference Docs

- Full documentation: [CONTRACTS.md](./CONTRACTS.md)
- Implementation details: [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)
- Soroban docs: https://soroban.stellar.org
- Stellar testnet explorer: https://stellar.expert/explorer/testnet

---

**Pro Tip**: Use `soroban contract invoke --help` to see all available parameters for any contract function.
