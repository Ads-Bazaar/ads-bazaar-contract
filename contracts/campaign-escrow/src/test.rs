//! Baseline tests covering what's actually implemented so far
//! (`initialize` and the read-only getters). As contributors fill in the
//! `todo!()` bodies in `lib.rs`, add corresponding tests here — e.g.
//! `test_create_and_fund_campaign`, `test_release_payment_pays_creator_minus_fee`.
#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup(
    env: &Env,
) -> (CampaignEscrowContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CampaignEscrowContract, ());
    let client = CampaignEscrowContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let dispute_contract = Address::generate(env);
    (client, contract_id, admin, dispute_contract)
}

#[test]
fn initialize_sets_admin_and_fee() {
    let env = Env::default();
    let (client, _contract_id, admin, dispute_contract) = setup(&env);

    client.initialize(&admin, &dispute_contract, &250);
}

#[test]
fn initialize_twice_fails() {
    let env = Env::default();
    let (client, _contract_id, admin, dispute_contract) = setup(&env);

    client.initialize(&admin, &dispute_contract, &250);
    let result = client.try_initialize(&admin, &dispute_contract, &250);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn initialize_rejects_out_of_range_fee() {
    let env = Env::default();
    let (client, _contract_id, admin, dispute_contract) = setup(&env);

    let result = client.try_initialize(
        &admin,
        &dispute_contract,
        &(ads_bazaar_shared::BASIS_POINTS_DENOMINATOR + 1),
    );
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn get_campaign_not_found_before_creation() {
    let env = Env::default();
    let (client, _contract_id, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let result = client.try_get_campaign(&0);
    assert_eq!(result, Err(Ok(Error::CampaignNotFound)));
}

#[test]
fn initialize_bumps_instance_ttl() {
    let env = Env::default();
    let (client, contract_id, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let admin_back = env.as_contract(&contract_id, || super::storage::get_admin(&env));
    assert_eq!(admin_back, Ok(admin));
}

#[test]
fn set_and_get_campaign_maintains_ttl() {
    let env = Env::default();
    let (client, contract_id, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let business = Address::generate(&env);
    let token = Address::generate(&env);
    let campaign = Campaign {
        id: 1,
        business: business.clone(),
        asset: ads_bazaar_shared::PayoutAsset {
            token,
            symbol: String::from_str(&env, "USDC"),
        },
        total_budget: 1_000_000,
        escrow_balance: 0,
        max_creators: 5,
        approved_count: 0,
        status: ads_bazaar_shared::CampaignStatus::Draft,
        application_deadline: env.ledger().timestamp() + 86_400,
        completion_deadline: env.ledger().timestamp() + 604_800,
        metadata_uri: String::from_str(&env, "ipfs://campaign"),
    };

    env.as_contract(&contract_id, || {
        let _ = super::storage::set_campaign(&env, &campaign);
        let loaded = super::storage::get_campaign(&env, 1).unwrap();
        assert_eq!(loaded.id, campaign.id);
        assert_eq!(loaded.business, campaign.business);
    });
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn create_campaign_is_not_yet_implemented() {
    // Documents current scaffold state: this will start failing (in a good
    // way) once `create_campaign` is implemented — replace this test with a
    // real assertion at that point.
    let env = Env::default();
    let (client, _contract_id, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let business = Address::generate(&env);
    let token = Address::generate(&env);
    let asset = ads_bazaar_shared::PayoutAsset {
        token,
        symbol: String::from_str(&env, "USDC"),
    };

    client.create_campaign(
        &business,
        &asset,
        &1_000_000,
        &5,
        &(env.ledger().timestamp() + 86_400),
        &(env.ledger().timestamp() + 604_800),
        &String::from_str(&env, "ipfs://brief"),
    );
}
