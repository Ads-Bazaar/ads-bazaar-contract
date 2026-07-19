//! # ads-bazaar-dispute-resolution
//!
//! Arbitrates disputes over campaign payouts held by `campaign-escrow`. As
//! with that contract, this crate ships the data model, storage schema,
//! errors and public API surface; the arbitration workflow itself
//! (assigning arbiters, evidence windows, resolving outcomes and calling
//! back into escrow) is left as `todo!()` for contributors — the
//! arbitration *model* (single trusted arbiter vs. staked jurors vs. an
//! oracle) is the biggest open design question in this repo.
#![no_std]

mod error;
mod events;
mod storage;
mod types;

pub use error::Error;
pub use types::Dispute;

use ads_bazaar_shared::{CampaignId, DisputeId, DisputeOutcome};
use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct DisputeResolutionContract;

#[contractimpl]
impl DisputeResolutionContract {
    /// One-time setup. `escrow_contract` should be the deployed
    /// `campaign-escrow` contract's address — this contract will need to
    /// call back into it (`resolve_dispute_payout`) once resolution logic
    /// is implemented.
    pub fn initialize(env: Env, admin: Address, escrow_contract: Address) -> Result<(), Error> {
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();

        storage::set_admin(&env, &admin);
        storage::set_escrow_contract(&env, &escrow_contract);
        storage::bump_instance(&env);
        Ok(())
    }

    /// Raise a dispute over a creator's payout on a given campaign.
    ///
    /// TODO(contributors): implement. Should call
    /// `campaign_escrow::Client::freeze_for_dispute` on the configured
    /// escrow contract once that hook exists, so funds can't be released
    /// mid-dispute. Decide who may raise a dispute (business, creator, or
    /// both) and whether there's a time window after proof submission.
    #[allow(unused_variables)]
    pub fn raise_dispute(
        env: Env,
        raised_by: Address,
        campaign_id: CampaignId,
        creator: Address,
        reason_uri: String,
    ) -> Result<DisputeId, Error> {
        storage::bump_instance(&env);
        raised_by.require_auth();
        todo!("design + implement dispute raising — see doc comment above")
    }

    /// Assign an arbiter to review a raised dispute.
    ///
    /// TODO(contributors): implement once the arbitration model is decided.
    #[allow(unused_variables)]
    pub fn assign_arbiter(
        env: Env,
        admin: Address,
        dispute_id: DisputeId,
        arbiter: Address,
    ) -> Result<(), Error> {
        storage::bump_instance(&env);
        admin.require_auth();
        todo!("design + implement arbiter assignment — see doc comment above")
    }

    /// Arbiter resolves a dispute with a final outcome, then calls back
    /// into `campaign-escrow::resolve_dispute_payout` to release/refund
    /// the frozen funds accordingly.
    ///
    /// TODO(contributors): implement.
    #[allow(unused_variables)]
    pub fn resolve_dispute(
        env: Env,
        arbiter: Address,
        dispute_id: DisputeId,
        outcome: DisputeOutcome,
    ) -> Result<(), Error> {
        storage::bump_instance(&env);
        arbiter.require_auth();
        todo!("design + implement dispute resolution — see doc comment above")
    }

    /// Read-only lookup of a dispute's current state.
    pub fn get_dispute(env: Env, dispute_id: DisputeId) -> Result<Dispute, Error> {
        storage::bump_instance(&env);
        storage::get_dispute(&env, dispute_id)
    }
}

#[cfg(test)]
mod test;
