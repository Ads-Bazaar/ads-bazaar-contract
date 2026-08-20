//! Client for the narrow slice of `dispute-resolution` that this contract calls.
//!
//! Declared locally with `#[contractclient]` rather than depending on the
//! `ads-bazaar-dispute-resolution` crate: linking that crate into this one would
//! pull its `#[contractimpl]` exports into this contract's wasm, so both
//! contracts' entry points would ship in a single binary. Keep these
//! signatures in sync with `dispute-resolution/src/lib.rs`. This mirrors the
//! local `escrow` client in `dispute-resolution/src/escrow.rs`.
//!
//! The method is declared infallible even though the dispute-resolution
//! contract returns `Result<_, Error>`. The encoding is identical on success,
//! and an error from the callee traps the whole invocation — which is the
//! behavior we want: `resolve_dispute` must not settle a payout while leaving
//! dispute-resolution's record behind as permanently open.
#![allow(dead_code)]

use ads_bazaar_shared::{CampaignId, DisputeOutcome};
use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "DisputeResolutionClient")]
pub trait DisputeContract {
    fn close_dispute(
        env: Env,
        caller: Address,
        campaign_id: CampaignId,
        creator: Address,
        outcome: DisputeOutcome,
    );
}
