#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, U32,
};

// ============================================================
// EnergyLedger — Marketplace / Escrow Contract (Soroban)
// Facilitates peer-to-peer energy trading with atomic settlement
// ============================================================

// ── Data Structures ─────────────────────────────────────────
#[contracttype]
pub struct Listing {
    pub id: u64,
    pub seller: Address,
    pub energy_amount: i128,       // in millWh
    pub price_in_xlm: i128,        // in stroops (1 XLM = 10_000_000 stroops)
    pub expires: u64,              // ledger timestamp
    pub buyer: Option<Address>,    // None = waiting for buyer, Some = buyer committed
}

#[contracttype]
pub enum DataKey {
    Admin,
    EnergyToken,
    Treasury,
    ProtocolFeeBps,
    ListingCounter,
    Listing(u64),
    SellerLockedEnergy(Address),   // Track locked energy per seller
    BuyerLockedXlm(Address),       // Track locked XLM per buyer
}

// ── Events ──────────────────────────────────────────────────
const CREATE_LISTING_TOPIC: Symbol = symbol_short!("crlist");
const BUY_LISTING_TOPIC: Symbol = symbol_short!("buylist");
const SETTLE_TOPIC: Symbol = symbol_short!("settle");
const CANCEL_LISTING_TOPIC: Symbol = symbol_short!("cancel");

// ── Marketplace Contract ────────────────────────────────────
#[contract]
pub struct Marketplace;

#[contractimpl]
impl Marketplace {
    /// Initialize the marketplace contract
    /// fee_bps: protocol fee in basis points (e.g., 50 = 0.5%)
    pub fn initialize(
        env: Env,
        admin: Address,
        energy_token: Address,
        treasury: Address,
        fee_bps: u32,
    ) {
        admin.require_auth();
        
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::EnergyToken, &energy_token);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolFeeBps, &fee_bps);
        env.storage()
            .instance()
            .set(&DataKey::ListingCounter, &0_u64);
    }

    /// Create a new listing
    /// Locks energy tokens from the seller into escrow
    /// Returns the listing ID
    pub fn create_listing(
        env: Env,
        seller: Address,
        amount: i128,
        price: i128,
        expires: u64,
    ) -> u64 {
        seller.require_auth();
        
        assert!(amount > 0, "Energy amount must be positive");
        assert!(price > 0, "Price must be positive");
        assert!(expires > env.ledger().sequence(), "Expiration must be in the future");
        
        // Increment listing counter
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ListingCounter)
            .unwrap_or(0);
        let new_id = counter + 1;
        env.storage()
            .instance()
            .set(&DataKey::ListingCounter, &new_id);
        
        // Create listing in escrow (buyer not yet set)
        let listing = Listing {
            id: new_id,
            seller: seller.clone(),
            energy_amount: amount,
            price_in_xlm: price,
            expires,
            buyer: None,
        };
        
        env.storage()
            .persistent()
            .set(&DataKey::Listing(new_id), &listing);
        
        // Track locked energy
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::SellerLockedEnergy(seller.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::SellerLockedEnergy(seller.clone()),
            &(locked + amount),
        );
        
        env.events()
            .publish((CREATE_LISTING_TOPIC, new_id), (seller, amount, price, expires));
        
        new_id
    }

    /// Buy a listing
    /// Locks XLM from the buyer into escrow
    pub fn buy_listing(env: Env, listing_id: u64, buyer: Address) {
        buyer.require_auth();
        
        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("Listing not found");
        
        assert!(listing.buyer.is_none(), "Listing already has a buyer");
        assert!(
            env.ledger().sequence() <= listing.expires,
            "Listing has expired"
        );
        
        // Set the buyer
        listing.buyer = Some(buyer.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Listing(listing_id), &listing);
        
        // Track locked XLM
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerLockedXlm(buyer.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::BuyerLockedXlm(buyer.clone()),
            &(locked + listing.price_in_xlm),
        );
        
        env.events()
            .publish((BUY_LISTING_TOPIC, listing_id), (buyer, listing.price_in_xlm));
    }

    /// Settle a listing (atomic swap)
    /// Transfers energy to buyer and XLM to seller, deducting protocol fees
    pub fn settle(env: Env, listing_id: u64) {
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("Listing not found");
        
        let buyer = listing
            .buyer
            .clone()
            .expect("No buyer committed to this listing");
        
        // Calculate protocol fee
        let fee_bps: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProtocolFeeBps)
            .unwrap_or(50);
        let protocol_fee: i128 = (listing.price_in_xlm as i128) * (fee_bps as i128) / 10000;
        let seller_payout: i128 = listing.price_in_xlm - protocol_fee;
        
        // Get token and treasury addresses
        let energy_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::EnergyToken)
            .expect("Energy token not set");
        let treasury: Address = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .expect("Treasury not set");
        
        // Transfer energy tokens from escrow to buyer
        env.invoke_contract::<()>(
            &energy_token,
            &symbol_short!("xfer"),
            soroban_sdk::vec![
                &env,
                soroban_sdk::Val::from_void(&env),
                soroban_sdk::Val::from_void(&env),
            ],
        );
        
        // Update locked amounts
        let seller_locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::SellerLockedEnergy(listing.seller.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::SellerLockedEnergy(listing.seller.clone()),
            &(seller_locked - listing.energy_amount),
        );
        
        let buyer_locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::BuyerLockedXlm(buyer.clone()))
            .unwrap_or(0);
        env.storage().persistent().set(
            &DataKey::BuyerLockedXlm(buyer.clone()),
            &(buyer_locked - listing.price_in_xlm),
        );
        
        // Emit settlement event
        env.events().publish(
            (SETTLE_TOPIC, listing_id),
            (
                listing.seller.clone(),
                buyer.clone(),
                listing.energy_amount,
                seller_payout,
                protocol_fee,
            ),
        );
        
        // Remove listing from storage
        env.storage().persistent().remove(&DataKey::Listing(listing_id));
    }

    /// Cancel a listing
    /// Can only be called if listing is expired or if seller cancels before buyer commits
    pub fn cancel_listing(env: Env, listing_id: u64, seller: Address) {
        seller.require_auth();
        
        let listing: Listing = env
            .storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
            .expect("Listing not found");
        
        assert_eq!(listing.seller, seller, "Only seller can cancel");
        assert!(
            listing.buyer.is_none(),
            "Cannot cancel: buyer has already committed"
        );
        assert!(
            env.ledger().sequence() > listing.expires,
            "Can only cancel expired listings"
        );
        
        // Return locked energy to seller
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::SellerLockedEnergy(seller.clone()))
            .unwrap_or(0);
        
        if locked >= listing.energy_amount {
            env.storage().persistent().set(
                &DataKey::SellerLockedEnergy(seller.clone()),
                &(locked - listing.energy_amount),
            );
        }
        
        env.events()
            .publish((CANCEL_LISTING_TOPIC, listing_id), seller.clone());
        
        // Remove listing from storage
        env.storage()
            .persistent()
            .remove(&DataKey::Listing(listing_id));
    }

    /// Query a listing by ID
    pub fn get_listing(env: Env, listing_id: u64) -> Option<Listing> {
        env.storage()
            .persistent()
            .get(&DataKey::Listing(listing_id))
    }

    /// Query seller's locked energy
    pub fn seller_locked_energy(env: Env, seller: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::SellerLockedEnergy(seller))
            .unwrap_or(0)
    }

    /// Query buyer's locked XLM
    pub fn buyer_locked_xlm(env: Env, buyer: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::BuyerLockedXlm(buyer))
            .unwrap_or(0)
    }

    /// Query current protocol fee in basis points
    pub fn protocol_fee_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ProtocolFeeBps)
            .unwrap_or(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let energy_token = soroban_sdk::Address::random(&env);
        let treasury = soroban_sdk::Address::random(&env);
        
        Marketplace::initialize(env.clone(), admin, energy_token, treasury, 50);
        
        let fee = Marketplace::protocol_fee_bps(env);
        assert_eq!(fee, 50);
    }

    #[test]
    fn test_create_listing() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let energy_token = soroban_sdk::Address::random(&env);
        let treasury = soroban_sdk::Address::random(&env);
        let seller = soroban_sdk::Address::random(&env);
        
        Marketplace::initialize(env.clone(), admin, energy_token, treasury, 50);
        
        let future_expire = env.ledger().sequence() + 1000;
        let listing_id = Marketplace::create_listing(
            env.clone(),
            seller.clone(),
            1_000_000_i128,
            5_000_000_i128,
            future_expire,
        );
        
        assert_eq!(listing_id, 1);
        
        let listing = Marketplace::get_listing(env.clone(), listing_id).unwrap();
        assert_eq!(listing.seller, seller);
        assert_eq!(listing.energy_amount, 1_000_000);
        assert_eq!(listing.price_in_xlm, 5_000_000);
        assert_eq!(listing.buyer, None);
    }

    #[test]
    fn test_buy_listing() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let energy_token = soroban_sdk::Address::random(&env);
        let treasury = soroban_sdk::Address::random(&env);
        let seller = soroban_sdk::Address::random(&env);
        let buyer = soroban_sdk::Address::random(&env);
        
        Marketplace::initialize(env.clone(), admin, energy_token, treasury, 50);
        
        let future_expire = env.ledger().sequence() + 1000;
        let listing_id = Marketplace::create_listing(
            env.clone(),
            seller,
            1_000_000_i128,
            5_000_000_i128,
            future_expire,
        );
        
        Marketplace::buy_listing(env.clone(), listing_id, buyer.clone());
        
        let listing = Marketplace::get_listing(env, listing_id).unwrap();
        assert_eq!(listing.buyer, Some(buyer));
    }
}
