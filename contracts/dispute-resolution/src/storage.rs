#![allow(dead_code)]

use ads_bazaar_shared::DisputeId;
use soroban_sdk::{contracttype, Address, Env};

use crate::error::Error;
use crate::types::Dispute;

const LEDGER_BUMP: u32 = 535_680;
const LEDGER_THRESHOLD: u32 = 500_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    EscrowContract,
    NextDisputeId,
    Dispute(DisputeId),
}

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

pub fn set_escrow_contract(env: &Env, escrow_contract: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::EscrowContract, escrow_contract);
}

pub fn get_escrow_contract(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::EscrowContract)
        .ok_or(Error::NotInitialized)
}

pub fn next_dispute_id(env: &Env) -> DisputeId {
    let id: DisputeId = env
        .storage()
        .instance()
        .get(&DataKey::NextDisputeId)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::NextDisputeId, &(id + 1));
    id
}

pub fn get_dispute(env: &Env, id: DisputeId) -> Result<Dispute, Error> {
    let key = DataKey::Dispute(id);
    let dispute = env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::DisputeNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    Ok(dispute)
}

pub fn set_dispute(env: &Env, id: DisputeId, dispute: &Dispute) {
    let key = DataKey::Dispute(id);
    env.storage().persistent().set(&key, dispute);
    env.storage().persistent().extend_ttl(
        &key,
        LEDGER_THRESHOLD,
        LEDGER_BUMP,
    );
}
