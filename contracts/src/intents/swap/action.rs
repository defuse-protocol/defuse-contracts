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
    /// NOTE: This MUST not exist before.
    pub id: IntentId,

    /// Desired asset as an output.
    pub asset_out: AssetWithAccount,

    /// Lockup period when initiator cannot rollback the intent.  
    /// MUST come before `expiration` if specified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockup_until: Option<Deadline>,

    /// Deadline to execute the swap.  
    /// Note that the intent can still be rolled back at any time
    /// unless `lockup_until` is specified.
    pub expiration: Deadline,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referral: Option<AccountId>,
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub struct ExecuteSwapIntentAction {
    /// Unique ID of the intent.
    /// NOTE: This MUST exist.
    pub id: IntentId,

    /// Where to send `asset_in` to
    pub recipient: GenericAccount,
}
