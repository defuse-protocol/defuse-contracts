use near_sdk::{near, AccountId};
use serde_with::{serde_as, DefaultOnNull};

use super::{Asset, Deadline, IntentId};

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case")]
pub enum SwapIntentAction {
    Create(CreateSwapIntentAction),
    Execute(ExecuteSwapIntentAction),
}

#[derive(Debug, Clone)]
#[serde_as]
#[near(serializers = [json, borsh])]
pub struct CreateSwapIntentAction {
    /// This should not exist before
    pub id: IntentId,
    /// Desired asset as an output
    pub asset_out: Asset,
    /// Where to send asset_out.
    /// By default: back to sender
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull")]
    pub recipient: Option<AccountId>,
    /// After deadline can not be executed and can be rollbacked
    pub deadline: Deadline,
}

#[derive(Debug, Clone)]
#[serde_as]
#[near(serializers = [json, borsh])]
pub struct ExecuteSwapIntentAction {
    pub id: IntentId,
    /// By default: back to sender
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull")]
    pub recipient: Option<AccountId>,
}
