#![no_std]

mod error;
mod events;
mod storage;
mod types;

pub use error::Error;
pub use types::{Application, Campaign, ProtocolConfig};

use ads_bazaar_shared::{CampaignId, PayoutAsset};
use soroban_sdk::{contract, contractimpl, Address, Env, String};

fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
    admin.require_auth();
    let stored_admin = storage::get_admin(env)?;
    if stored_admin != *admin {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    if storage::get_paused(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

#[contract]
pub struct CampaignEscrowContract;

#[contractimpl]
impl CampaignEscrowContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        dispute_contract: Address,
        fee_bps: i128,
    ) -> Result<(), Error> {
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        if !(0..=ads_bazaar_shared::BASIS_POINTS_DENOMINATOR).contains(&fee_bps) {
            return Err(Error::InvalidAmount);
        }
        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_treasury(&env, &admin);
        storage::set_dispute_contract(&env, &dispute_contract);
        storage::set_fee_bps(&env, fee_bps);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, true);
        events::ContractPaused { admin }.publish(&env);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        storage::set_paused(&env, false);
        events::ContractUnpaused { admin }.publish(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::get_paused(&env)
    }

    #[allow(unused_variables, clippy::too_many_arguments)]
    pub fn create_campaign(
        env: Env,
        business: Address,
        asset: PayoutAsset,
        total_budget: i128,
        max_creators: u32,
        application_deadline: u64,
        completion_deadline: u64,
        metadata_uri: String,
    ) -> Result<CampaignId, Error> {
        require_not_paused(&env)?;
        todo!("design + implement campaign creation")
    }

    #[allow(unused_variables)]
    pub fn fund_campaign(
        env: Env,
        business: Address,
        campaign_id: CampaignId,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        business.require_auth();
        todo!("design + implement escrow funding")
    }

    #[allow(unused_variables)]
    pub fn apply_to_campaign(
        env: Env,
        creator: Address,
        campaign_id: CampaignId,
        pitch_uri: String,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        creator.require_auth();
        todo!("design + implement creator applications")
    }

    #[allow(unused_variables)]
    pub fn approve_creator(
        env: Env,
        business: Address,
        campaign_id: CampaignId,
        creator: Address,
        payout_amount: i128,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        business.require_auth();
        todo!("design + implement creator approval")
    }

    #[allow(unused_variables)]
    pub fn submit_proof(
        env: Env,
        creator: Address,
        campaign_id: CampaignId,
        proof_uri: String,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        creator.require_auth();
        todo!("design + implement proof submission/verification")
    }

    #[allow(unused_variables)]
    pub fn release_payment(
        env: Env,
        business: Address,
        campaign_id: CampaignId,
        creator: Address,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        business.require_auth();
        todo!("design + implement payout release")
    }

    #[allow(unused_variables)]
    pub fn cancel_campaign(
        env: Env,
        business: Address,
        campaign_id: CampaignId,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        business.require_auth();
        todo!("design + implement cancellation/refund")
    }

    #[allow(unused_variables)]
    pub fn freeze_for_dispute(
        env: Env,
        campaign_id: CampaignId,
        creator: Address,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        todo!("design + implement dispute freeze hook")
    }

    #[allow(unused_variables)]
    pub fn resolve_dispute_payout(
        env: Env,
        campaign_id: CampaignId,
        creator: Address,
        creator_bps: i128,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        todo!("design + implement dispute payout resolution")
    }

    pub fn get_campaign(env: Env, campaign_id: CampaignId) -> Result<Campaign, Error> {
        storage::get_campaign(&env, campaign_id)
    }

    pub fn get_application(
        env: Env,
        campaign_id: CampaignId,
        creator: Address,
    ) -> Result<Application, Error> {
        storage::get_application(&env, campaign_id, &creator)
    }

    pub fn get_protocol_config(env: Env) -> Result<ProtocolConfig, Error> {
        let admin = storage::get_admin(&env)?;
        let treasury = storage::get_treasury(&env)?;
        let fee_bps = storage::get_fee_bps(&env)?;

        storage::extend_instance_ttl(&env);

        Ok(ProtocolConfig {
            admin,
            treasury,
            fee_bps,
        })
    }
}

#[cfg(test)]
mod test;
