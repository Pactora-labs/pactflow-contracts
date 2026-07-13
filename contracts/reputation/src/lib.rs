#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, BytesN, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReputation {
    pub completed_projects: u32,
    pub failed_projects: u32,
    pub rating_sum: u32,
    pub review_hashes: Vec<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Reputation(Address),
}

#[contract]
pub struct ReputationContract;

#[contractimpl]
impl ReputationContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn record_completion(env: Env, user: Address, rating: u32, review_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if rating > 5 {
            panic!("rating must be between 0 and 5");
        }

        let mut rep = Self::get_reputation(env.clone(), user.clone());
        rep.completed_projects += 1;
        rep.rating_sum += rating;
        rep.review_hashes.push_back(review_hash);

        env.storage().instance().set(&DataKey::Reputation(user), &rep);
    }

    pub fn record_failure(env: Env, user: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut rep = Self::get_reputation(env.clone(), user.clone());
        rep.failed_projects += 1;

        env.storage().instance().set(&DataKey::Reputation(user), &rep);
    }

    pub fn get_reputation(env: Env, user: Address) -> UserReputation {
        env.storage().instance().get(&DataKey::Reputation(user.clone())).unwrap_or(UserReputation {
            completed_projects: 0,
            failed_projects: 0,
            rating_sum: 0,
            review_hashes: Vec::new(&env),
        })
    }

    pub fn calculate_trust_score(env: Env, user: Address) -> u32 {
        let rep = Self::get_reputation(env, user);
        let total = rep.completed_projects + rep.failed_projects;
        if total == 0 {
            return 100;
        }

        let success_rate = (rep.completed_projects * 100) / total;
        let avg_rating = if rep.completed_projects > 0 {
            rep.rating_sum / rep.completed_projects
        } else {
            0
        };

        (success_rate * avg_rating) / 5
    }
}
