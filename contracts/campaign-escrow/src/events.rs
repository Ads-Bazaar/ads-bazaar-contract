#![allow(dead_code)]

use ads_bazaar_shared::CampaignId;
use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug)]
pub struct CampaignCreated {
    #[topic]
    pub business: Address,
    pub campaign_id: CampaignId,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CampaignFunded {
    #[topic]
    pub campaign_id: CampaignId,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CreatorApplied {
    #[topic]
    pub campaign_id: CampaignId,
    #[topic]
    pub creator: Address,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CreatorApproved {
    #[topic]
    pub campaign_id: CampaignId,
    #[topic]
    pub creator: Address,
    pub payout_amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ProofSubmitted {
    #[topic]
    pub campaign_id: CampaignId,
    #[topic]
    pub creator: Address,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct PaymentReleased {
    #[topic]
    pub campaign_id: CampaignId,
    #[topic]
    pub creator: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct CampaignCancelled {
    #[topic]
    pub campaign_id: CampaignId,
    pub refunded_amount: i128,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ContractPaused {
    #[topic]
    pub admin: Address,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ContractUnpaused {
    #[topic]
    pub admin: Address,
}
