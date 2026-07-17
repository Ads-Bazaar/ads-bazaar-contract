//! Baseline tests covering what's actually implemented so far
//! (`initialize` and the read-only getters). As contributors fill in the
//! `todo!()` bodies in `lib.rs`, add corresponding tests here — e.g.
//! `test_create_and_fund_campaign`, `test_release_payment_pays_creator_minus_fee`.
#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup(env: &Env) -> (CampaignEscrowContractClient<'_>, Address, Address) {
    let contract_id = env.register(CampaignEscrowContract, ());
    let client = CampaignEscrowContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let dispute_contract = Address::generate(env);
    (client, admin, dispute_contract)
}

#[test]
fn initialize_sets_admin_and_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);

    client.initialize(&admin, &dispute_contract, &250);
}

#[test]
fn initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);

    client.initialize(&admin, &dispute_contract, &250);
    let result = client.try_initialize(&admin, &dispute_contract, &250);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn initialize_rejects_out_of_range_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);

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
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let result = client.try_get_campaign(&0);
    assert_eq!(result, Err(Ok(Error::CampaignNotFound)));
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn create_campaign_is_not_yet_implemented() {
    // Documents current scaffold state: this will start failing (in a good
    // way) once `create_campaign` is implemented — replace this test with a
    // real assertion at that point.
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);
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

#[test]
fn test_cancel_campaign_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let business = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_addr = env.register().stellar_asset_contract(token_admin);
    let token = soroban_sdk::token::Client::new(&env, &token_addr);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_addr);

    token_admin_client.mint(&client.address, &1_000_000);

    let asset = ads_bazaar_shared::PayoutAsset {
        token: token_addr,
        symbol: String::from_str(&env, "USDC"),
    };

    let campaign_id = 1;
    let campaign = Campaign {
        id: campaign_id,
        business: business.clone(),
        asset,
        total_budget: 1_000_000,
        escrow_balance: 1_000_000,
        max_creators: 5,
        approved_count: 0,
        application_deadline: env.ledger().timestamp() + 86_400,
        completion_deadline: env.ledger().timestamp() + 604_800,
        metadata_uri: String::from_str(&env, "ipfs://brief"),
        status: ads_bazaar_shared::CampaignStatus::Funded,
    };
    
    env.as_contract(&client.address, || {
        storage::set_campaign(&env, &campaign);
    });

    assert_eq!(token.balance(&business), 0);
    assert_eq!(token.balance(&client.address), 1_000_000);

    client.cancel_campaign(&business, &campaign_id);

    let updated_campaign = client.get_campaign(&campaign_id);
    assert_eq!(updated_campaign.status, ads_bazaar_shared::CampaignStatus::Cancelled);
    assert_eq!(updated_campaign.escrow_balance, 0);

    assert_eq!(token.balance(&business), 1_000_000);
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_cancel_campaign_non_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, dispute_contract) = setup(&env);
    client.initialize(&admin, &dispute_contract, &250);

    let business = Address::generate(&env);
    let malicious = Address::generate(&env);
    let token_addr = Address::generate(&env);

    let asset = ads_bazaar_shared::PayoutAsset {
        token: token_addr,
        symbol: String::from_str(&env, "USDC"),
    };

    let campaign_id = 1;
    let campaign = Campaign {
        id: campaign_id,
        business: business.clone(),
        asset,
        total_budget: 1_000_000,
        escrow_balance: 1_000_000,
        max_creators: 5,
        approved_count: 0,
        application_deadline: env.ledger().timestamp() + 86_400,
        completion_deadline: env.ledger().timestamp() + 604_800,
        metadata_uri: String::from_str(&env, "ipfs://brief"),
        status: ads_bazaar_shared::CampaignStatus::Funded,
    };
    
    env.as_contract(&client.address, || {
        storage::set_campaign(&env, &campaign);
    });

    let res = client.try_cancel_campaign(&malicious, &campaign_id);
    assert_eq!(res, Err(Ok(Error::NotCampaignOwner)));
}
