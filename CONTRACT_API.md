# EnergyLedger Contract API Specification

Complete reference for all contract functions, events, and data structures.

## Energy Token Contract API

### Contract Address
Store as environment variable: `ENERGY_TOKEN_ID`

### Functions

#### `initialize(env: Env, admin: Address)`
Initialize the contract with an administrator address.

**Parameters:**
- `admin` (Address): Administrator who can register producers

**Authorization:** `admin` must sign the transaction

**Effects:**
- Sets admin for the contract
- Initializes retired energy counter to 0
- Emits event: `(regprod, admin)`

**Example:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  -- initialize \
  --admin GXXXXXX...
```

---

#### `register_producer(env: Env, admin: Address, producer: Address)`
Authorize a new energy producer to mint tokens.

**Parameters:**
- `admin` (Address): Contract administrator
- `producer` (Address): Producer to authorize

**Authorization:** `admin` must sign the transaction

**Validation:**
- Only the stored admin can register producers
- Fails if `admin` is not the contract admin

**Effects:**
- Adds producer to registry
- Emits event: `(regprod, producer)`

**Example:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  -- register_producer \
  --admin GXXXXXX... \
  --producer GYYYYYY...
```

---

#### `mint(env: Env, to: Address, amount: i128) → void`
Mint new energy tokens (producer only).

**Parameters:**
- `to` (Address): Recipient address (must be producer)
- `amount` (i128): Amount to mint in millWh

**Authorization:** `to` (producer) must sign the transaction

**Validation:**
- `amount` must be positive (> 0)
- `to` must be a registered producer
- Fails if `to` is not registered

**Effects:**
- Adds `amount` to `to`'s balance
- Emits event: `(mint, to)` with amount

**Gas:** ~50 KB

**Example:**
```bash
# Mint 1000 kWh (1,000,000 millWh)
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source bob \
  -- mint \
  --to GBBBBB... \
  --amount 1000000
```

---

#### `transfer(env: Env, from: Address, to: Address, amount: i128) → void`
Transfer energy credits between addresses.

**Parameters:**
- `from` (Address): Sender address
- `to` (Address): Recipient address
- `amount` (i128): Amount to transfer in millWh

**Authorization:** `from` must sign the transaction

**Validation:**
- `amount` must be positive (> 0)
- `from` must have balance ≥ `amount`
- Fails if insufficient balance

**Effects:**
- Decrements `from`'s balance by `amount`
- Increments `to`'s balance by `amount`
- Emits event: `(xfer, from, to)` with amount

**Gas:** ~55 KB

**Example:**
```bash
# Transfer 500 kWh (500,000 millWh)
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source bob \
  -- transfer \
  --from GBBBBB... \
  --to GCCCCC... \
  --amount 500000
```

---

#### `balance(env: Env, address: Address) → i128`
Query the energy credit balance for an address.

**Parameters:**
- `address` (Address): Address to query

**Returns:** Balance in millWh (i128)

**Authorization:** None required (read-only)

**Gas:** ~5 KB

**Example:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  -- balance \
  --address GBBBBB...

# Output: 1000000
```

---

#### `retire(env: Env, from: Address, amount: i128) → void`
Retire (burn) energy credits permanently.

Represents final energy consumption for carbon offset accounting. Removes tokens from circulation permanently.

**Parameters:**
- `from` (Address): Address retiring the energy
- `amount` (i128): Amount to retire in millWh

**Authorization:** `from` must sign the transaction

**Validation:**
- `amount` must be positive (> 0)
- `from` must have balance ≥ `amount`
- Fails if insufficient balance

**Effects:**
- Decrements `from`'s balance by `amount`
- Increments global `RetiredTotal` by `amount`
- Emits event: `(retire, from)` with amount

**Gas:** ~60 KB

**Example:**
```bash
# Retire 250 kWh (250,000 millWh) for carbon offset
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source bob \
  -- retire \
  --from GBBBBB... \
  --amount 250000
```

---

#### `total_retired(env: Env) → i128`
Query total energy retired globally.

Used for carbon offset impact metrics and statistics.

**Returns:** Total retired energy in millWh (i128)

**Authorization:** None required (read-only)

**Gas:** ~5 KB

**Example:**
```bash
soroban contract invoke \
  --id $ENERGY_TOKEN_ID \
  --source alice \
  -- total_retired

# Output: 500000
```

---

### Events

| Event | Topics | Data | Emitted When |
|-------|--------|------|--------------|
| `mint` | `(mint, producer_address)` | amount | Producer mints energy |
| `xfer` | `(xfer, from_address, to_address)` | amount | Energy transferred |
| `retire` | `(retire, address)` | amount | Energy burned |
| `regprod` | `(regprod, producer_address)` | producer_address | Producer registered |

---

## Marketplace Contract API

### Contract Address
Store as environment variable: `MARKETPLACE_ID`

### Data Types

#### `Listing`
```rust
{
  id: u64,                    // Unique listing ID
  seller: Address,            // Seller address
  energy_amount: i128,        // Energy amount in millWh
  price_in_xlm: i128,         // Price in stroops
  expires: u64,               // Expiration ledger sequence
  buyer: Option<Address>      // Buyer address (None if waiting)
}
```

---

### Functions

#### `initialize(env: Env, admin: Address, energy_token: Address, treasury: Address, fee_bps: u32) → void`
Initialize the marketplace contract.

**Parameters:**
- `admin` (Address): Contract administrator
- `energy_token` (Address): Energy token contract address
- `treasury` (Address): Address for protocol fee collection
- `fee_bps` (u32): Protocol fee in basis points (50 = 0.5%)

**Authorization:** `admin` must sign the transaction

**Effects:**
- Sets up contract configuration
- Initializes listing counter to 0

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- initialize \
  --admin GXXXXXX... \
  --energy_token $ENERGY_TOKEN_ID \
  --treasury GZZZZZ... \
  --fee_bps 50
```

---

#### `create_listing(env: Env, seller: Address, amount: i128, price: i128, expires: u64) → u64`
Create a new energy listing.

Locks energy tokens in escrow. Seller cannot withdraw until cancelled or settled.

**Parameters:**
- `seller` (Address): Seller address
- `amount` (i128): Energy amount to sell in millWh
- `price` (i128): Sale price in stroops
- `expires` (u64): Expiration ledger sequence

**Authorization:** `seller` must sign the transaction

**Validation:**
- `amount` > 0 (positive energy)
- `price` > 0 (positive price)
- `expires` > current ledger sequence (future expiration)

**Returns:** Listing ID (u64)

**Effects:**
- Increments listing counter
- Creates listing with no buyer yet
- Locks `amount` in seller's escrow
- Emits event: `(crlist, listing_id)`

**Gas:** ~70 KB

**Example:**
```bash
# Sell 500 kWh (500,000 millWh) for 2.5 XLM (2,500,000 stroops)
# Expires in 1000 ledgers (~1 hour on testnet)
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source bob \
  -- create_listing \
  --seller GBBBBB... \
  --amount 500000 \
  --price 2500000 \
  --expires 1000
```

---

#### `buy_listing(env: Env, listing_id: u64, buyer: Address) → void`
Commit to buy a listing.

Locks buyer's XLM in escrow. Buyer cannot withdraw until settled or listing cancelled.

**Parameters:**
- `listing_id` (u64): ID of listing to purchase
- `buyer` (Address): Buyer address

**Authorization:** `buyer` must sign the transaction

**Validation:**
- Listing must exist
- Listing must not already have a buyer
- Listing must not be expired (current ledger ≤ expiration)

**Effects:**
- Sets buyer on listing
- Locks `price_in_xlm` in buyer's escrow
- Emits event: `(buylist, listing_id)`

**Gas:** ~60 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source charlie \
  -- buy_listing \
  --listing_id 1 \
  --buyer GCCCC...
```

---

#### `settle(env: Env, listing_id: u64) → void`
Execute atomic settlement of a listing.

Swaps energy tokens from seller to buyer and XLM from buyer to seller with protocol fees.

**Parameters:**
- `listing_id` (u64): ID of listing to settle

**Authorization:** None required

**Validation:**
- Listing must exist
- Listing must have a buyer
- Sufficient balances on both sides

**Effects:**
- Transfers `energy_amount` from seller to buyer
- Transfers `price * (1 - fee_bps/10000)` XLM to seller
- Transfers `price * (fee_bps/10000)` XLM to treasury
- Deletes listing
- Emits event: `(settle, listing_id)`

**Gas:** ~100 KB

**Settlement Example:**
```
Listing: 500,000 millWh for 2,500,000 stroops
Fee: 50 basis points (0.5%)

Settlement:
  - Seller receives: 2,500,000 * 0.995 = 2,487,500 stroops
  - Treasury receives: 2,500,000 * 0.005 = 12,500 stroops
  - Buyer receives: 500,000 millWh
```

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- settle \
  --listing_id 1
```

---

#### `cancel_listing(env: Env, listing_id: u64, seller: Address) → void`
Cancel an expired listing.

Returns locked energy to seller. Can only be called after expiration.

**Parameters:**
- `listing_id` (u64): ID of listing to cancel
- `seller` (Address): Seller address

**Authorization:** `seller` must sign the transaction

**Validation:**
- Listing must exist
- Must be called by original seller
- Listing must NOT have a buyer
- Must be expired (current ledger > expiration)

**Effects:**
- Returns `energy_amount` to seller's balance
- Removes listing from storage
- Emits event: `(cancel, listing_id)`

**Gas:** ~50 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source bob \
  -- cancel_listing \
  --listing_id 1 \
  --seller GBBBBB...
```

---

#### `get_listing(env: Env, listing_id: u64) → Option<Listing>`
Query a listing by ID.

**Parameters:**
- `listing_id` (u64): Listing to query

**Returns:** Listing data or null if not found

**Authorization:** None required (read-only)

**Gas:** ~10 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- get_listing \
  --listing_id 1

# Output (JSON-like):
# {
#   id: 1,
#   seller: GBBBBB...,
#   energy_amount: 500000,
#   price_in_xlm: 2500000,
#   expires: 1000,
#   buyer: GCCCC...
# }
```

---

#### `seller_locked_energy(env: Env, seller: Address) → i128`
Query total locked energy for a seller.

Sum of all energy in active listings for the seller.

**Parameters:**
- `seller` (Address): Seller address

**Returns:** Total locked energy in millWh

**Authorization:** None required (read-only)

**Gas:** ~10 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- seller_locked_energy \
  --seller GBBBBB...

# Output: 500000
```

---

#### `buyer_locked_xlm(env: Env, buyer: Address) → i128`
Query total locked XLM for a buyer.

Sum of all XLM in active listings for the buyer.

**Parameters:**
- `buyer` (Address): Buyer address

**Returns:** Total locked XLM in stroops

**Authorization:** None required (read-only)

**Gas:** ~10 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- buyer_locked_xlm \
  --buyer GCCCC...

# Output: 2500000
```

---

#### `protocol_fee_bps(env: Env) → u32`
Query the current protocol fee percentage.

**Returns:** Fee in basis points (50 = 0.5%)

**Authorization:** None required (read-only)

**Gas:** ~5 KB

**Example:**
```bash
soroban contract invoke \
  --id $MARKETPLACE_ID \
  --source alice \
  -- protocol_fee_bps

# Output: 50
```

---

### Events

| Event | Topics | Data | Emitted When |
|-------|--------|------|--------------|
| `crlist` | `(crlist, listing_id)` | (seller, amount, price, expires) | Listing created |
| `buylist` | `(buylist, listing_id)` | (buyer, price) | Buyer committed |
| `settle` | `(settle, listing_id)` | (seller, buyer, energy, seller_payout, fee) | Settlement executed |
| `cancel` | `(cancel, listing_id)` | seller | Listing cancelled |

---

## Units Reference

### Energy
- **millWh (millwatt-hours)**
- 1 kWh = 1,000 millWh
- 1 MWh = 1,000,000 millWh
- Range: 0 to 9,223,372,036,854,775,807 millWh (i128 max)

### Currency
- **stroops (Stellar lumens)**
- 1 XLM = 10,000,000 stroops
- Range: 0 to 9,223,372,036 XLM (i128 max)

---

## Error Codes

| Error | Cause |
|-------|-------|
| "Amount must be positive" | Tried to transfer/mint/retire 0 or negative amount |
| "Insufficient balance" | Sender has less balance than requested transfer |
| "Not a registered producer" | Non-producer tried to mint tokens |
| "Unauthorized: caller is not admin" | Non-admin tried to register producer |
| "Admin not set" | Contract not initialized |
| "Listing not found" | Listing ID doesn't exist |
| "No buyer committed to this listing" | Tried to settle listing without buyer |
| "Listing already has a buyer" | Tried to buy already-bought listing |
| "Listing has expired" | Tried to buy/interact with expired listing |
| "Can only cancel expired listings" | Tried to cancel non-expired listing |
| "Only seller can cancel" | Non-seller tried to cancel listing |

---

## Gas Estimates

| Operation | Gas | Notes |
|-----------|-----|-------|
| initialize (token) | 50 KB | One-time setup |
| register_producer | 40 KB | Per producer |
| mint | 50 KB | Per transaction |
| transfer | 55 KB | Per transaction |
| balance (query) | 5 KB | Read-only |
| retire | 60 KB | Per transaction |
| initialize (marketplace) | 50 KB | One-time setup |
| create_listing | 70 KB | Per listing |
| buy_listing | 60 KB | Per buyer |
| settle | 100 KB | Full atomic swap |
| cancel_listing | 50 KB | Per cancellation |
| query functions | 5-10 KB | Read-only |

---

## Best Practices

1. **Always Verify Amounts**: Check energy and price before creating listings
2. **Use Reasonable Expirations**: Set expires 100+ ledgers in future (~10+ minutes on testnet)
3. **Monitor Events**: Subscribe to contract events for real-time updates
4. **Handle Authorization**: Ensure callers are ready to sign transactions
5. **Cache Queries**: Cache balance/listing queries with short TTL to reduce gas
6. **Batch Operations**: Combine multiple operations when possible (future enhancement)
7. **Error Handling**: Implement retry logic for failed transactions

---

## TypeScript Integration

See `src/lib/soroban-integration.ts` for TypeScript wrapper classes and utilities for calling these contract functions from the frontend.

---

**API Version**: 1.0.0  
**Last Updated**: July 2026  
**Soroban SDK**: 21.0
