#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, BytesN, Vec, Map};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeDecision {
    None = 0,
    ClientWins = 1,
    FreelancerWins = 2,
    SplitSplit = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub id: u32,
    pub escrow_contract: Address,
    pub client: Address,
    pub freelancer: Address,
    pub reason_hash: BytesN<32>,
    pub client_evidence: Vec<BytesN<32>>,
    pub freelancer_evidence: Vec<BytesN<32>>,
    pub client_votes: u32,
    pub freelancer_votes: u32,
    pub split_votes: u32,
    pub is_resolved: bool,
    pub final_decision: DisputeDecision,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Arbiters,
    Disputes,
    DisputeCounter,
}

#[contract]
pub struct ArbitrationContract;

#[contractimpl]
impl ArbitrationContract {
    pub fn initialize(env: Env, admin: Address, arbiters: Vec<Address>) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Arbiters, &arbiters);
        env.storage().instance().set(&DataKey::Disputes, &Map::<u32, Dispute>::new(&env));
        env.storage().instance().set(&DataKey::DisputeCounter, &0u32);
    }

    pub fn raise_dispute(
        env: Env,
        escrow_contract: Address,
        client: Address,
        freelancer: Address,
        reason_hash: BytesN<32>,
    ) -> u32 {
        let mut disputes: Map<u32, Dispute> = env.storage().instance().get(&DataKey::Disputes).unwrap();
        let mut counter: u32 = env.storage().instance().get(&DataKey::DisputeCounter).unwrap();

        counter += 1;

        let dispute = Dispute {
            id: counter,
            escrow_contract,
            client,
            freelancer,
            reason_hash,
            client_evidence: Vec::new(&env),
            freelancer_evidence: Vec::new(&env),
            client_votes: 0,
            freelancer_votes: 0,
            split_votes: 0,
            is_resolved: false,
            final_decision: DisputeDecision::None,
        };

        disputes.set(counter, dispute);
        env.storage().instance().set(&DataKey::Disputes, &disputes);
        env.storage().instance().set(&DataKey::DisputeCounter, &counter);

        counter
    }

    pub fn submit_evidence(env: Env, dispute_id: u32, submitter: Address, evidence_hash: BytesN<32>) {
        submitter.require_auth();

        let mut disputes: Map<u32, Dispute> = env.storage().instance().get(&DataKey::Disputes).unwrap();
        let mut dispute = disputes.get(dispute_id).unwrap();

        if dispute.is_resolved {
            panic!("dispute is already resolved");
        }

        if submitter == dispute.client {
            dispute.client_evidence.push_back(evidence_hash);
        } else if submitter == dispute.freelancer {
            dispute.freelancer_evidence.push_back(evidence_hash);
        } else {
            panic!("not a party in this dispute");
        }

        disputes.set(dispute_id, dispute);
        env.storage().instance().set(&DataKey::Disputes, &disputes);
    }

    pub fn vote(env: Env, dispute_id: u32, arbiter: Address, decision: DisputeDecision) {
        arbiter.require_auth();

        let arbiters: Vec<Address> = env.storage().instance().get(&DataKey::Arbiters).unwrap();
        if !arbiters.contains(arbiter) {
            panic!("not authorized arbiter");
        }

        let mut disputes: Map<u32, Dispute> = env.storage().instance().get(&DataKey::Disputes).unwrap();
        let mut dispute = disputes.get(dispute_id).unwrap();

        if dispute.is_resolved {
            panic!("dispute is already resolved");
        }

        match decision {
            DisputeDecision::ClientWins => dispute.client_votes += 1,
            DisputeDecision::FreelancerWins => dispute.freelancer_votes += 1,
            DisputeDecision::SplitSplit => dispute.split_votes += 1,
            DisputeDecision::None => panic!("invalid decision"),
        }

        let total_votes = dispute.client_votes + dispute.freelancer_votes + dispute.split_votes;
        let quorum = arbiters.len() / 2 + 1;

        if total_votes >= quorum as u32 {
            dispute.is_resolved = true;
            if dispute.client_votes > dispute.freelancer_votes && dispute.client_votes > dispute.split_votes {
                dispute.final_decision = DisputeDecision::ClientWins;
            } else if dispute.freelancer_votes > dispute.client_votes && dispute.freelancer_votes > dispute.split_votes {
                dispute.final_decision = DisputeDecision::FreelancerWins;
            } else {
                dispute.final_decision = DisputeDecision::SplitSplit;
            }
        }

        disputes.set(dispute_id, dispute);
        env.storage().instance().set(&DataKey::Disputes, &disputes);
    }

    pub fn get_dispute(env: Env, dispute_id: u32) -> Dispute {
        let disputes: Map<u32, Dispute> = env.storage().instance().get(&DataKey::Disputes).unwrap();
        disputes.get(dispute_id).unwrap()
    }
}
