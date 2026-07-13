use ads_bazaar_shared::{CampaignId, DisputeOutcome, DisputeStatus};
use soroban_sdk::{contracttype, Address, String};

/// A dispute raised over a single creator's payout within a campaign.
///
/// TODO(contributors): the arbitration model itself (single trusted arbiter
/// vs. staked juror voting vs. an oracle) is an open design question — this
/// struct assumes a single `arbiter` for now as the simplest starting point.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub campaign_id: CampaignId,
    pub creator: Address,
    pub raised_by: Address,
    pub reason_uri: String,
    pub arbiter: Option<Address>,
    pub status: DisputeStatus,
    /// `DisputeOutcome::Pending` until `resolve_dispute` sets a final outcome.
    pub outcome: DisputeOutcome,
    pub raised_at: u64,
    pub resolved_at: Option<u64>,
}
