use near_sdk::{near, AccountId};

use super::{AssetWithAccount, Deadline, GenericAccount, IntentId};

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SwapIntentAction {
    Create(CreateSwapIntentAction),
    Execute(ExecuteSwapIntentAction),
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub struct CreateSwapIntentAction {
    /// Unique ID of intent.  
    /// NOTE: Intent with such id MUST NOT exist before.
    pub id: IntentId,

    /// Desired output asset and its recipient.
    pub asset_out: AssetWithAccount,

    /// Lockup period when initiator cannot rollback the intent.  
    /// MUST come before `expiration` if specified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockup_until: Option<Deadline>,

    /// Deadline to execute the swap.  
    /// Note that the intent can still be rolled back at any time
    /// unless [`lockup_until`] is specified.
    pub expiration: Deadline,

    /// Referral
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referral: Option<AccountId>,
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub struct ExecuteSwapIntentAction {
    /// Unique ID of the intent.  
    /// NOTE: Intent with such id MUST exist.
    pub id: IntentId,

    /// Optional proof for cross-chain assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,

    /// Recipient for [`asset_in`](super::SwapIntent::asset_in)
    pub recipient: GenericAccount,
}
