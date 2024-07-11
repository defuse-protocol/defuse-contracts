use near_sdk::{near, AccountId};

use super::{Asset, Expiration, IntentId};

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case")]
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
    pub asset_out: Asset,
    /// Where to send asset_out. By default: back to initiator.
    #[serde(default)]
    pub recipient: Option<AccountId>,
    /// Deadline to execute the swap.
    /// NOTE: intent can still be rollbacked at any time.
    pub expiration: Expiration,
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub struct ExecuteSwapIntentAction {
    /// Unique ID of the intent.
    /// NOTE: This MUST exist.
    pub id: IntentId,
    /// Where to send asset_in. By default: back to executor.
    #[serde(default)]
    pub recipient: Option<AccountId>,
}
