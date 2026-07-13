//! Baseline tests covering what's actually implemented so far
//! (`initialize` and `get_dispute`). Add tests alongside each `todo!()` as
//! it gets implemented in `lib.rs`.
#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup(env: &Env) -> (DisputeResolutionContractClient<'_>, Address, Address) {
    let contract_id = env.register(DisputeResolutionContract, ());
    let client = DisputeResolutionContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let escrow_contract = Address::generate(env);
    (client, admin, escrow_contract)
}

#[test]
fn initialize_sets_admin_and_escrow_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, escrow_contract) = setup(&env);

    client.initialize(&admin, &escrow_contract);
}

#[test]
fn initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, escrow_contract) = setup(&env);

    client.initialize(&admin, &escrow_contract);
    let result = client.try_initialize(&admin, &escrow_contract);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn get_dispute_not_found_before_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, escrow_contract) = setup(&env);
    client.initialize(&admin, &escrow_contract);

    let result = client.try_get_dispute(&0);
    assert_eq!(result, Err(Ok(Error::DisputeNotFound)));
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn raise_dispute_is_not_yet_implemented() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, escrow_contract) = setup(&env);
    client.initialize(&admin, &escrow_contract);

    let raised_by = Address::generate(&env);
    let creator = Address::generate(&env);
    client.raise_dispute(
        &raised_by,
        &0,
        &creator,
        &String::from_str(&env, "ipfs://evidence"),
    );
}
