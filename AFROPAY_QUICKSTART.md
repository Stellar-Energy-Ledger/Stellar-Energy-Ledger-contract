# AfroPay-Stellar Remittance — Quick Start Guide

Deploy and test the AfroPay cross-border remittance contract in 15 minutes.

---

## ⚡ Quick Deploy (5 minutes)

### 1. Build the Contract

```bash
soroban contract build
```

Output: `.wasm` file in `target/wasm32-unknown-unknown/release/afropay_remittance.wasm`

### 2. Set Environment

```bash
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export ADMIN_KEY="SXXXXXX..."  # Your admin secret key
export ORACLE_KEY="SXXXXXX..." # Oracle secret key
```

### 3. Deploy

```bash
AFROPAY=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/afropay_remittance.wasm \
  --source alice \
  --network testnet | tail -1)

echo "Contract ID: $AFROPAY"
```

### 4. Initialize

```bash
ADMIN=$(soroban keys address alice)
ORACLE=$(soroban keys address bob)

soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $ADMIN \
  --oracle $ORACLE
```

---

## 🧪 Test Flow (5 minutes)

### 1. Create Remittance

```bash
SENDER=$(soroban keys address alice)
RECIPIENT=$(soroban keys address bob)
TOKEN_ADDR="CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4"  # USDC testnet

# Create verification hash (in real scenario, hash of bank receipt)
HASH="0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

TX_ID=$(soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- create_remittance \
  --sender $SENDER \
  --recipient $RECIPIENT \
  --token $TOKEN_ADDR \
  --amount 100000000 \
  --verification_hash $HASH \
  --lock_time 1000 | tail -1)

echo "Remittance TX_ID: $TX_ID"
```

### 2. Query Remittance

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- get_remittance \
  --tx_id $TX_ID
```

Expected output:
```
{
  sender: alice,
  recipient: bob,
  amount: 100000000,
  status: 0,        // Pending
  expires_at: 1000,
  ...
}
```

### 3. Release Funds (Oracle)

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source bob \
  --network testnet \
  -- release_funds \
  --tx_id $TX_ID \
  --proof_receipt $HASH
```

### 4. Verify Completion

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- get_status \
  --tx_id $TX_ID
```

Expected output: `1` (Completed)

---

## 💡 Common Operations

### Create Remittance (500 USDC)

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- create_remittance \
  --sender $SENDER \
  --recipient $RECIPIENT \
  --token $TOKEN_ADDR \
  --amount 500000000 \
  --verification_hash $HASH \
  --lock_time 2000
```

### Update Oracle

```bash
NEW_ORACLE=$(soroban keys address charlie)

soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- update_oracle \
  --new_oracle $NEW_ORACLE
```

### Claim Refund (after expiration)

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- claim_refund \
  --tx_id $TX_ID
```

### Query Oracle

```bash
soroban contract invoke \
  --id $AFROPAY \
  --source alice \
  --network testnet \
  -- get_oracle
```

---

## 🔧 Advanced: Local Testing

### Run Unit Tests

```bash
cargo test
```

Tests included:
- ✅ `test_initialize` - Contract setup
- ✅ `test_create_remittance` - Escrow locking
- ✅ `test_release_funds` - Oracle verification
- ✅ `test_claim_refund` - Expiration handling
- ✅ `test_update_oracle` - Admin operations

---

## 📱 TypeScript Integration

### Install Dependencies

```bash
npm install @stellar/stellar-sdk
```

### Create Remittance (Node.js)

```typescript
import { AfroPay, usdcToStroops } from './afropay';

const afropay = new AfroPay('<AFROPAY_CONTRACT_ID>');

// Create verification hash
const hash = Buffer.from('0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f', 'hex');

// Create remittance for 500 USDC
const txId = await afropay.createRemittance(
  senderKeypair,
  recipientAddress,
  usdcAddress,
  usdcToStroops(500),
  hash,
  2000  // 2000 ledgers
);

console.log(`Remittance: ${txId}`);
```

### Query Remittance

```typescript
const remittance = await afropay.getRemittance(BigInt(txId));

if (remittance) {
  console.log(`Status: ${['Pending', 'Completed', 'Refunded'][remittance.status]}`);
  console.log(`Amount: ${stroopsToUsdc(remittance.amount)} USDC`);
}
```

---

## 🆘 Troubleshooting

### "Contract not found"
- Verify contract ID is correct
- Check network (testnet vs mainnet)
- Confirm deployment succeeded

### "Verification proof does not match"
- Ensure proof_receipt matches verification_hash
- Hash must be exactly same (case-sensitive)
- Use same format: BytesN<32>

### "Remittance has not expired yet"
- Cannot refund before expiration
- Wait for expiration time or contact oracle
- Check current ledger vs expires_at

### "Authorization failed"
- Verify correct keypair is being used
- Sender must sign create_remittance
- Oracle must sign release_funds
- Admin must sign update_oracle

### "Insufficient balance"
- Ensure account has XLM for fees
- Get testnet XLM from [friendbot](https://friendbot.stellar.org)
- Wait for previous transactions to confirm

---

## 📊 Reference

### Contract Address
Store the contract ID:
```bash
echo $AFROPAY > .afropay-contract
```

Later:
```bash
AFROPAY=$(cat .afropay-contract)
```

### Status Codes

| Code | Name | Meaning |
|------|------|---------|
| 0 | Pending | Awaiting oracle verification |
| 1 | Completed | Funds released successfully |
| 2 | Refunded | Funds returned to sender |

### Amount Examples

```
100 USDC    = 100,000,000 stroops
500 USDC    = 500,000,000 stroops
1000 USDC   = 1,000,000,000 stroops
```

### Lock Time Examples

```
~5 minutes   = 100 ledgers
~1 hour      = 1,000 ledgers  (typical)
~1 day       = 24,000 ledgers
```

---

## ✅ Verification Checklist

- [ ] Contract builds without errors
- [ ] Contract deployed to testnet
- [ ] Contract initialized with admin and oracle
- [ ] Remittance created successfully
- [ ] Status query returns "Pending"
- [ ] Oracle releases funds with matching proof
- [ ] Status query returns "Completed"
- [ ] Events emitted correctly
- [ ] Refund works after expiration
- [ ] Oracle can be updated by admin

---

## 📚 Next Steps

1. **Explore**: Read [AFROPAY_SPEC.md](./AFROPAY_SPEC.md) for full API
2. **Integrate**: Use [afropay.ts](./afropay.ts) for frontend
3. **Test**: Modify [afropay_remittance.rs](./afropay_remittance.rs) for custom logic
4. **Deploy**: Mainnet deployment when ready

---

## 🎯 What's Next?

### Build Frontend UI
- Remittance form (sender, recipient, amount)
- Status dashboard
- Verification upload
- Refund interface

### Set Up Oracle Service
- Bank API integration
- Wallet verification
- Hash computation
- Proof submission

### Configure Off-Ramps
- Multiple currencies (NGN, GHS, KES, etc.)
- Fiat receiver networks
- Settlement timing

### Deploy to Mainnet
- Security audit completed
- Liquidity pools established
- Oracle infrastructure live
- Full compliance in place

---

**Time to Production**: ~2 weeks (after initial testing and audit)

---

**Status**: ✅ Ready for Testing  
**Version**: 1.0.0  
**Network**: Stellar Testnet  
**Soroban SDK**: v21.0
