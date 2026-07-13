#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Client,
    Freelancer,
    Token,
    TotalAmount,
    ReleasedAmount,
    MilestoneCount,
    IsCancelled,
    IsFunded,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    pub fn initialize(
        env: Env,
        client: Address,
        freelancer: Address,
        token: Address,
        total_amount: i128,
        milestone_count: u32,
    ) {
        if env.storage().instance().has(&DataKey::Client) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalAmount, &total_amount);
        env.storage().instance().set(&DataKey::ReleasedAmount, &0i128);
        env.storage().instance().set(&DataKey::MilestoneCount, &milestone_count);
        env.storage().instance().set(&DataKey::IsCancelled, &false);
        env.storage().instance().set(&DataKey::IsFunded, &false);
    }

    pub fn fund(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let total_amount: i128 = env.storage().instance().get(&DataKey::TotalAmount).unwrap();
        let is_funded: bool = env.storage().instance().get(&DataKey::IsFunded).unwrap();

        if is_funded {
            panic!("already funded");
        }

        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&client, &env.current_contract_address(), &total_amount);

        env.storage().instance().set(&DataKey::IsFunded, &true);
    }

    pub fn release_payment(env: Env, amount: i128) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let is_cancelled: bool = env.storage().instance().get(&DataKey::IsCancelled).unwrap();
        if is_cancelled {
            panic!("contract is cancelled");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        let total_amount: i128 = env.storage().instance().get(&DataKey::TotalAmount).unwrap();
        let released: i128 = env.storage().instance().get(&DataKey::ReleasedAmount).unwrap();

        let new_released = released + amount;
        if new_released > total_amount {
            panic!("amount exceeds total budget");
        }

        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &freelancer, &amount);

        env.storage().instance().set(&DataKey::ReleasedAmount, &new_released);
    }

    pub fn refund_client(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        
        freelancer.require_auth();

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let total_amount: i128 = env.storage().instance().get(&DataKey::TotalAmount).unwrap();
        let released: i128 = env.storage().instance().get(&DataKey::ReleasedAmount).unwrap();

        let remaining = total_amount - released;
        if remaining <= 0 {
            panic!("no funds to refund");
        }

        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &client, &remaining);

        env.storage().instance().set(&DataKey::ReleasedAmount, &total_amount);
        env.storage().instance().set(&DataKey::IsCancelled, &true);
    }

    pub fn cancel(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let is_funded: bool = env.storage().instance().get(&DataKey::IsFunded).unwrap();
        if !is_funded {
            env.storage().instance().set(&DataKey::IsCancelled, &true);
            return;
        }

        panic!("cannot cancel once funded without mutual agreement or dispute");
    }
}
