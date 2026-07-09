# AfroPay-Stellar Remittance Contract Specification

Complete specification for the AfroPay cross-border remittance platform on Soroban.

---

## 📋 Overview

AfroPay-Stellar is a cross-border remittance platform built on the Stellar blockchain using Soroban smart contracts. It enables users to send stablecoins (USDC) across borders with local currency settlement verified by an oracle.

**Key Features:**
- Escrow-based remittance locking
- Oracle-verified fiat settlement
- Automatic refunds on expiration
- Cryptographic verification proof
- Event logging for all transactions

---

## 🏗️ Architecture

### Smart Contract Components

```
┌─────────────────────────────────────────┐
│   AfroPay Remittance Contract           │
├─────────────────────────────────────────┤
│ • Escrow Management                     │
│ • Oracle Verification                   │
│ • Refund Processing                     │
│ • Event Emission                        │
└─────────────────────────────────────────┘
         │              │              │
         ↓              ↓              ↓
    [Sender]       [Oracle]      [Recipient]
   (Locks USDC) (Verifies Fiat) (Receives USDC)
```

### Remittance Flow

```
1. SENDER INITIATES
   └─ create_remittance()
      Locks USDC in escrow
      Stores verification hash
      Sets expiration time

2. ORACLE VERIFIES
   └─ Checks bank/mobile wallet delivery
      Verifies fiat currency receipt

3. ORACLE RELEASES
   └─ release_funds()
      Provides matching verification proof
      Transfers USDC to recipient

4. COMPLETION or REFUND
   ├─ Successful: USDC received by recipient
   └─ Expired: Sender can claim_refund()
```

---

## 📊 Data Structures

### Remittance Struct

```rust
struct Remittance {
    sender: Address,               // Remittance originator
    recipient: Address,            // Recipient or off-ramp agent
    amount: i128,                  // Stablecoin amount (stroops)
    token: Address,                // Token contract (e.g., USDC)
    verification_hash: BytesN<32>, // SHA256 of receipt/transaction ID
    status: u32,                   // 0=Pending, 1=Completed, 2=Refunded
    expires_at: u64,               // Ledger timestamp deadline
    created_at: u64,               // Creation timestamp (block height)
}
```

### Storage Keys

```rust
enum DataKey {
    Admin,                           // Admin address
    Oracle,                          // Oracle address
    RemittanceCounter,               // Next transaction ID
    Remittance(u64),                // tx_id -> Remittance
    SenderRemittances(Address),     // sender -> [tx_ids]
    RecipientRemittances(Address),  // recipient -> [tx_ids]
}
```

### Status Values

| Value | Name | Meaning |
|-------|------|---------|
| 0 | Pending | Awaiting oracle verification |
| 1 | Completed | Funds released to recipient |
| 2 | Refunded | Funds returned to sender |

---

## ⚙️ Contract Functions

### initialize(env: Env, admin: Address, oracle: Address) → void

Initialize the contract with admin and oracle addresses.

**Parameters:**
- `admin` (Address): Contract administrator
- `oracle` (Address): Trusted verification oracle

**Authorization:** `admin` must sign

**Effects:**
- Sets admin for the contract
- Sets oracle for fiat verification
- Initializes remittance counter to 0

**Events:** Emits `OracleUpdated` event

---

### create_remittance(...) → u64

Create a new remittance transaction with escrow.

**Signature:**
```rust
pub fn create_remittance(
    env: Env,
    sender: Address,
    recipient: Address,
    token: Address,
    amount: i128,
    verification_hash: BytesN<32>,
    lock_time: u64
) -> u64
```

**Parameters:**
- `sender` (Address): Remittance originator
- `recipient` (Address): Final recipient or off-ramp agent
- `token` (Address): Stablecoin contract address
- `amount` (i128): Amount in stroops (USDC × 1,000,000)
- `verification_hash` (BytesN<32>): SHA256 hash of receipt
- `lock_time` (u64): Lock duration in ledger sequences

**Authorization:** `sender` must sign

**Validation:**
- `amount` > 0
- `lock_time` > 0
- Expiration calculated as current_ledger + lock_time

**Returns:** Transaction ID (u64)

**Effects:**
- Increments remittance counter
- Creates Remittance struct with status = Pending
- Stores remittance in persistent storage
- Locks funds in escrow

**Events:** Emits `RemittanceCreated` with (sender, recipient, amount, expires_at)

**Gas:** ~70 KB

---

### release_funds(env: Env, tx_id: u64, proof_receipt: BytesN<32>) → void

Release funds from escrow upon oracle verification.

**Parameters:**
- `tx_id` (u64): Transaction ID to release
- `proof_receipt` (BytesN<32>): Proof matching verification_hash

**Authorization:** `oracle` must sign

**Validation:**
- Remittance exists
- Status is Pending
- Proof matches verification_hash

**Effects:**
- Updates status to Completed
- Transfers stablecoins to recipient
- Stores updated remittance

**Events:** Emits `RemittanceReleased` with (sender, recipient, amount)

**Gas:** ~80 KB

---

### claim_refund(env: Env, tx_id: u64) → void

Claim refund for expired pending remittance.

**Parameters:**
- `tx_id` (u64): Transaction ID to refund

**Authorization:** `sender` must sign

**Validation:**
- Remittance exists
- Status is Pending
- Current ledger > expiration time

**Effects:**
- Updates status to Refunded
- Returns stablecoins to sender
- Stores updated remittance

**Events:** Emits `RemittanceRefunded` with (sender, tx_id)

**Gas:** ~60 KB

---

### update_oracle(env: Env, new_oracle: Address) → void

Update the authorized oracle address.

**Parameters:**
- `new_oracle` (Address): New oracle address

**Authorization:** `admin` must sign

**Effects:**
- Updates oracle address in storage

**Events:** Emits `OracleUpdated` with new_oracle

**Gas:** ~20 KB

---

### get_remittance(env: Env, tx_id: u64) → Option<Remittance>

Query remittance details.

**Parameters:**
- `tx_id` (u64): Transaction ID

**Returns:** Remittance struct or None if not found

**Authorization:** None (read-only)

**Gas:** ~10 KB

---

### get_status(env: Env, tx_id: u64) → u32

Query remittance status.

**Parameters:**
- `tx_id` (u64): Transaction ID

**Returns:** Status value (0, 1, 2, or u32::MAX if not found)

**Authorization:** None (read-only)

**Gas:** ~5 KB

---

### get_oracle(env: Env) → Address

Query current oracle address.

**Returns:** Oracle Address

**Authorization:** None (read-only)

**Gas:** ~5 KB

---

### get_admin(env: Env) → Address

Query admin address.

**Returns:** Admin Address

**Authorization:** None (read-only)

**Gas:** ~5 KB

---

## 🎯 Events

### RemittanceCreated

**Topic:** `(remittance_created, tx_id)`  
**Data:** `(sender, recipient, amount, expires_at)`

Emitted when a new remittance is created and locked in escrow.

---

### RemittanceReleased

**Topic:** `(remittance_released, tx_id)`  
**Data:** `(sender, recipient, amount)`

Emitted when funds are released to recipient upon oracle verification.

---

### RemittanceRefunded

**Topic:** `(remittance_refunded, tx_id)`  
**Data:** `(sender)`

Emitted when sender claims refund after expiration.

---

### OracleUpdated

**Topic:** `(oracle_updated,)`  
**Data:** `(new_oracle_address)`

Emitted when oracle address is updated by admin.

---

## 🔐 Security Model

### Authorization

| Function | Required Auth | Why |
|----------|---------------|-----|
| initialize() | admin | Prevent unauthorized setup |
| create_remittance() | sender | Only sender can lock funds |
| release_funds() | oracle | Only oracle can verify & release |
| claim_refund() | sender | Only sender can refund |
| update_oracle() | admin | Prevent unauthorized oracle changes |

### Verification Mechanism

1. **Verification Hash**
   - SHA256 hash of bank receipt or mobile wallet transaction ID
   - Created off-chain and provided by sender
   - Immutable after remittance creation

2. **Oracle Verification**
   - Oracle independently verifies fiat delivery
   - Computes same hash from receipt
   - Submits hash as proof in release_funds()

3. **Proof Matching**
   - Contract verifies proof_receipt == verification_hash
   - Prevents mismatched settlements

### Protection Against

- **Double-spending**: Status prevents re-release
- **Unauthorized release**: Oracle signature required
- **Unauthorized refund**: Sender signature + expiration required
- **Orphaned funds**: Automatic refund after expiration

---

## 💰 Units & Precision

### Stablecoin (USDC)
- **Base unit**: stroops (6 decimals)
- **1 USDC** = 1,000,000 stroops
- **Storage type**: i128
- **Max value**: ~9,223,372,036 USDC

### Ledger Timestamps
- **Lock time**: Ledger sequences (blocks)
- **Typical duration**: 1,000 sequences ≈ 1 hour on testnet (4.5s/block)
- **Storage type**: u64

---

## 🧪 Testing Strategy

### Unit Tests

1. **Initialization**
   - Contract setup with admin and oracle
   - Verify stored addresses

2. **Create Remittance**
   - Lock USDC in escrow
   - Verify remittance struct
   - Check status = Pending

3. **Release Funds**
   - Oracle verification with matching proof
   - Update status to Completed
   - Verify event emission

4. **Claim Refund**
   - Expiration simulation
   - Status update to Refunded
   - Verify authorization

5. **Update Oracle**
   - Admin updates oracle
   - Verify new oracle address
   - Check event emission

### Test Scenarios

```rust
#[test]
fn test_create_remittance() { ... }

#[test]
fn test_release_funds_with_matching_proof() { ... }

#[test]
#[should_panic(expected = "Verification proof does not match")]
fn test_release_funds_with_wrong_proof() { ... }

#[test]
fn test_claim_refund_after_expiration() { ... }

#[test]
#[should_panic(expected = "Remittance has not expired yet")]
fn test_claim_refund_before_expiration() { ... }
```

---

## 📱 TypeScript Integration

### Creating a Remittance

```typescript
import { AfroPay, usdcToStroops, createVerificationHash } from './afropay';

const afropay = new AfroPay();

// Create verification hash from bank receipt
const receiptId = "NGN-2024-001234567";
const hash = await createVerificationHash(receiptId);

// Create remittance for 500 USDC
const txId = await afropay.createRemittance(
  senderKeypair,
  recipientAddress,
  usdcAddress,
  usdcToStroops(500),        // 500 × 1,000,000 stroops
  hash,                       // Verification proof
  1000                        // Lock for 1000 ledgers
);

console.log(`Remittance created: ${txId}`);
```

### Oracle Verification

```typescript
// Oracle checks bank delivery
const verified = await checkBankTransfer(receiptId);

if (verified) {
  // Release funds with matching proof
  const txHash = await afropay.releaseFunds(
    oracleKeypair,
    txId,
    hash  // Same hash from creation
  );
  console.log(`Funds released: ${txHash}`);
}
```

### Querying Status

```typescript
// Check remittance status
const remittance = await afropay.getRemittance(txId);

if (remittance) {
  console.log(`Status: ${remittance.status}`);
  console.log(`Amount: ${stroopsToUsdc(remittance.amount)} USDC`);
  console.log(`Expires: Block #${remittance.expires_at}`);
}
```

---

## 🚀 Deployment

### Prerequisites
- Rust 1.70+
- Soroban CLI
- Stellar testnet account with XLM

### Build
```bash
soroban contract build
```

### Deploy
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/afropay_remittance.wasm \
  --source alice \
  --network testnet
```

### Initialize
```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --oracle <ORACLE_ADDRESS>
```

---

## 📈 Gas Estimates

| Operation | Gas | Time |
|-----------|-----|------|
| initialize() | 30 KB | ~1s |
| create_remittance() | 70 KB | ~2s |
| release_funds() | 80 KB | ~2s |
| claim_refund() | 60 KB | ~2s |
| update_oracle() | 20 KB | ~1s |
| Query functions | 5-10 KB | <1s |

---

## 🔄 Typical Flow Timeline

```
T=0s:   Sender creates remittance (70 KB gas)
        └─ Status: Pending
        └─ Locked: USDC in contract

T=30m:  Oracle verifies bank delivery
        └─ Computes verification hash

T=31m:  Oracle releases funds (80 KB gas)
        └─ Status: Completed
        └─ USDC transferred to recipient

Timeline can vary:
- Quick release: 2-5 minutes (fast bank confirmation)
- Standard: 30 minutes - 2 hours (normal banking hours)
- Expiration refund: set by lock_time parameter
```

---

## 🛠️ Maintenance & Operations

### Admin Tasks

1. **Update Oracle**
   - When oracle organization changes
   - When oracle key is compromised
   - Use update_oracle() function

2. **Monitor Events**
   - Track remittance creation/completion
   - Identify failed transactions
   - Log for compliance

3. **Support Refunds**
   - Help users troubleshoot expired transactions
   - Verify sender identity before assisting

### Oracle Tasks

1. **Verify Deliveries**
   - Check bank/wallet confirmations
   - Verify fiat currency delivered
   - Create verification hash

2. **Release Funds**
   - Submit matching proof hash
   - Handle release_funds() calls
   - Log completions

3. **Error Handling**
   - Investigate proof mismatches
   - Retry releases if needed
   - Notify parties of failures

---

## 📋 Compliance & KYC

**Note**: This contract is the settlement layer. Compliance, KYC, and AML checks should be performed by:
- Off-ramp agents (receiving fiat)
- Remittance platform frontend
- Oracle verification service

The contract itself is compliance-agnostic and neutral.

---

## 🔗 Related Resources

- [Soroban Documentation](https://soroban.stellar.org)
- [Stellar Documentation](https://developers.stellar.org)
- [Contract API Reference](#contract-functions)
- [TypeScript SDK](./afropay.ts)

---

**Version**: 1.0.0  
**Status**: Production-Ready  
**Last Updated**: July 2026  
**Soroban SDK**: v21.0
