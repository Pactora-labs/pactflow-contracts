#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token, Vec};

/// Protocol fee in basis points (e.g. 100 = 1%)
const DEFAULT_FEE_BPS: u32 = 100;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    FeeBps,
    TotalCollected(Address), // keyed by token address
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeRecord {
    pub token: Address,
    pub amount: i128,
    pub from_contract: Address,
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    /// Initialize the treasury with an admin and optional custom fee BPS.
    pub fn initialize(env: Env, admin: Address, fee_bps: Option<u32>) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::FeeBps,
            &fee_bps.unwrap_or(DEFAULT_FEE_BPS),
        );
    }

    /// Collect a protocol fee from an escrow contract.
    /// The escrow contract calls this during payment release.
    pub fn collect_fee(env: Env, token_addr: Address, gross_amount: i128) -> i128 {
        let fee_bps: u32 = env.storage().instance().get(&DataKey::FeeBps).unwrap_or(DEFAULT_FEE_BPS);
        let fee = (gross_amount * fee_bps as i128) / 10_000;
        if fee <= 0 {
            return 0;
        }

        let token_client = token::Client::new(&env, &token_addr);
        // Fee is transferred by the caller (escrow contract) before this call
        token_client.transfer(&env.invoker(), &env.current_contract_address(), &fee);

        // Track total collected per token
        let key = DataKey::TotalCollected(token_addr);
        let prev: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(prev + fee));

        fee
    }

    /// Withdraw accumulated fees to a recipient address.
    pub fn withdraw(env: Env, token_addr: Address, amount: i128, recipient: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token_client = token::Client::new(&env, &token_addr);
        let contract_balance = token_client.balance(&env.current_contract_address());
        if amount > contract_balance {
            panic!("insufficient balance in treasury");
        }

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);
    }

    /// Get the current balance of a given token.
    pub fn get_balance(env: Env, token_addr: Address) -> i128 {
        let token_client = token::Client::new(&env, &token_addr);
        token_client.balance(&env.current_contract_address())
    }

    /// Get total protocol fees collected for a given token.
    pub fn get_total_collected(env: Env, token_addr: Address) -> i128 {
        env.storage().instance().get(&DataKey::TotalCollected(token_addr)).unwrap_or(0)
    }

    /// Update the protocol fee in basis points (admin only).
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if fee_bps > 1000 {
            panic!("fee cannot exceed 10%");
        }
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
    }

    /// Get current fee BPS.
    pub fn get_fee_bps(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::FeeBps).unwrap_or(DEFAULT_FEE_BPS)
    }

    /// Transfer admin role to a new address.
    pub fn update_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }
}
