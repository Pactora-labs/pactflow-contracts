#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, BytesN, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Rejected = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub id: u32,
    pub amount: i128,
    pub status: MilestoneStatus,
    pub deliverable_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    EscrowContract,
    Client,
    Freelancer,
    Milestones,
}

#[contract]
pub struct MilestoneContract;

#[contractimpl]
impl MilestoneContract {
    pub fn initialize(
        env: Env,
        escrow_contract: Address,
        client: Address,
        freelancer: Address,
        milestones: Vec<Milestone>,
    ) {
        if env.storage().instance().has(&DataKey::EscrowContract) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::EscrowContract, &escrow_contract);
        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Milestones, &milestones);
    }

    pub fn submit_milestone(env: Env, milestone_id: u32, hash: BytesN<32>) {
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        freelancer.require_auth();

        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        let mut found = false;

        for i in 0..milestones.len() {
            let mut m = milestones.get(i).unwrap();
            if m.id == milestone_id {
                if let MilestoneStatus::Pending = m.status {
                    m.status = MilestoneStatus::Submitted;
                    m.deliverable_hash = hash.clone();
                    milestones.set(i, m);
                    found = true;
                    break;
                } else if let MilestoneStatus::Rejected = m.status {
                    m.status = MilestoneStatus::Submitted;
                    m.deliverable_hash = hash.clone();
                    milestones.set(i, m);
                    found = true;
                    break;
                } else {
                    panic!("milestone not in pending or rejected status");
                }
            }
        }

        if !found {
            panic!("milestone not found");
        }

        env.storage().instance().set(&DataKey::Milestones, &milestones);
    }

    pub fn approve_milestone(env: Env, milestone_id: u32) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        let mut found = false;

        for i in 0..milestones.len() {
            let mut m = milestones.get(i).unwrap();
            if m.id == milestone_id {
                if let MilestoneStatus::Submitted = m.status {
                    m.status = MilestoneStatus::Approved;
                    milestones.set(i, m);
                    found = true;
                    break;
                } else {
                    panic!("milestone must be submitted before approval");
                }
            }
        }

        if !found {
            panic!("milestone not found");
        }

        env.storage().instance().set(&DataKey::Milestones, &milestones);
    }

    pub fn reject_milestone(env: Env, milestone_id: u32) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let mut milestones: Vec<Milestone> = env.storage().instance().get(&DataKey::Milestones).unwrap();
        let mut found = false;

        for i in 0..milestones.len() {
            let mut m = milestones.get(i).unwrap();
            if m.id == milestone_id {
                if let MilestoneStatus::Submitted = m.status {
                    m.status = MilestoneStatus::Rejected;
                    milestones.set(i, m);
                    found = true;
                    break;
                } else {
                    panic!("milestone must be submitted before rejection");
                }
            }
        }

        if !found {
            panic!("milestone not found");
        }

        env.storage().instance().set(&DataKey::Milestones, &milestones);
    }

    pub fn get_milestones(env: Env) -> Vec<Milestone> {
        env.storage().instance().get(&DataKey::Milestones).unwrap_or(Vec::new(&env))
    }
}
