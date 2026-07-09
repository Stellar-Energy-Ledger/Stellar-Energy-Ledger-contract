#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Symbol, symbol_short,
};

// ============================================================
// EnergyLedger — Energy Token Contract (Soroban)
// Handles tokenization of energy credits in millWh precision
// ============================================================

// ── Storage Keys ────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Balance(Address),          // millWh balance per address
    Admin,                     // contract admin
    Producer(Address),         // registered producers (persistent)
    RetiredTotal,              // total millWh retired globally
}

// ── Events ──────────────────────────────────────────────────
const MINT_TOPIC: Symbol = symbol_short!("mint");
const TRANSFER_TOPIC: Symbol = symbol_short!("xfer");
const RETIRE_TOPIC: Symbol = symbol_short!("retire");
const REGISTER_PRODUCER_TOPIC: Symbol = symbol_short!("regprod");

// ── Energy Token Contract ────────────────────────────────────
#[contract]
pub struct EnergyToken;

#[contractimpl]
impl EnergyToken {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::RetiredTotal, &0_i128);
    }

    /// Register a producer (admin only)
    /// Authorizes a producer address to mint energy tokens
    pub fn register_producer(env: Env, admin: Address, producer: Address) {
        admin.require_auth();
        
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        
        assert_eq!(admin, stored_admin, "Unauthorized: caller is not admin");
        
        env.storage()
            .persistent()
            .set(&DataKey::Producer(producer.clone()), &true);
        
        env.events()
            .publish((REGISTER_PRODUCER_TOPIC, producer.clone()), producer);
    }

    /// Mint energy tokens (producer only)
    /// amount is in millWh (milliwatt-hours)
    /// Only registered producers can mint
    pub fn mint(env: Env, to: Address, amount: i128) {
        to.require_auth();
        
        assert!(amount > 0, "Amount must be positive");
        
        let is_producer: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Producer(to.clone()))
            .unwrap_or(false);
        
        assert!(is_producer, "Unauthorized: caller is not a registered producer");
        
        let current: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(current + amount));
        
        env.events().publish((MINT_TOPIC, to.clone()), amount);
    }

    /// Transfer energy credits between addresses
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        
        assert!(amount > 0, "Amount must be positive");
        
        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        
        assert!(from_bal >= amount, "Insufficient balance");
        
        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(from_bal - amount));
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(to_bal + amount));
        
        env.events()
            .publish((TRANSFER_TOPIC, from.clone(), to.clone()), amount);
    }

    /// Query the energy credit balance for an address
    pub fn balance(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(address))
            .unwrap_or(0)
    }

    /// Retire (burn) consumed energy credits
    /// Represents final energy consumption and carbon offset accounting
    pub fn retire(env: Env, from: Address, amount: i128) {
        from.require_auth();
        
        assert!(amount > 0, "Amount must be positive");
        
        let bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        
        assert!(bal >= amount, "Insufficient balance to retire");
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(bal - amount));
        
        let retired: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RetiredTotal)
            .unwrap_or(0);
        
        env.storage()
            .instance()
            .set(&DataKey::RetiredTotal, &(retired + amount));
        
        env.events().publish((RETIRE_TOPIC, from.clone()), amount);
    }

    /// Query total retired credits globally
    /// Used for carbon offset impact metrics
    pub fn total_retired(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RetiredTotal)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as AddressTestUtils, Env as EnvTestUtils};

    #[test]
    fn test_initialize() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        
        EnergyToken::initialize(env.clone(), admin.clone());
        
        let stored_admin: soroban_sdk::Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();
        assert_eq!(admin, stored_admin);
    }

    #[test]
    fn test_register_and_mint() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let producer = soroban_sdk::Address::random(&env);
        
        EnergyToken::initialize(env.clone(), admin.clone());
        EnergyToken::register_producer(env.clone(), admin, producer.clone());
        
        let amount = 1_000_000_i128; // 1000 kWh in millWh
        EnergyToken::mint(env.clone(), producer.clone(), amount);
        
        let balance = EnergyToken::balance(env, producer);
        assert_eq!(balance, amount);
    }

    #[test]
    fn test_transfer() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let producer = soroban_sdk::Address::random(&env);
        let buyer = soroban_sdk::Address::random(&env);
        
        EnergyToken::initialize(env.clone(), admin.clone());
        EnergyToken::register_producer(env.clone(), admin, producer.clone());
        
        let mint_amount = 2_000_000_i128;
        EnergyToken::mint(env.clone(), producer.clone(), mint_amount);
        
        let transfer_amount = 500_000_i128;
        EnergyToken::transfer(
            env.clone(),
            producer.clone(),
            buyer.clone(),
            transfer_amount,
        );
        
        let producer_bal = EnergyToken::balance(env.clone(), producer);
        let buyer_bal = EnergyToken::balance(env, buyer);
        
        assert_eq!(producer_bal, mint_amount - transfer_amount);
        assert_eq!(buyer_bal, transfer_amount);
    }

    #[test]
    fn test_retire() {
        let env = soroban_sdk::Env::default();
        let admin = soroban_sdk::Address::random(&env);
        let producer = soroban_sdk::Address::random(&env);
        
        EnergyToken::initialize(env.clone(), admin.clone());
        EnergyToken::register_producer(env.clone(), admin, producer.clone());
        
        let amount = 1_000_000_i128;
        EnergyToken::mint(env.clone(), producer.clone(), amount);
        
        let retire_amount = 300_000_i128;
        EnergyToken::retire(env.clone(), producer.clone(), retire_amount);
        
        let balance = EnergyToken::balance(env.clone(), producer);
        let retired = EnergyToken::total_retired(env);
        
        assert_eq!(balance, amount - retire_amount);
        assert_eq!(retired, retire_amount);
    }
}
