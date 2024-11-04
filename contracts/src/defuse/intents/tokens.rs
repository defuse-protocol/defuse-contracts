use impl_tools::autoimpl;
use near_contract_standards::non_fungible_token;
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, NearToken};

use crate::{defuse::tokens::TokenId, nep245};

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct MtBatchTransfer {
    pub receiver_id: AccountId,
    pub token_ids: Vec<TokenId>,
    pub amounts: Vec<U128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.transfer)]
#[derive(Debug, Clone)]
pub struct MtBatchTransferCall {
    #[serde(flatten)]
    pub transfer: MtBatchTransfer,

    /// `msg` to pass in `mt_on_transfer`
    pub msg: String,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct FtWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub amount: U128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct NftWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id: non_fungible_token::TokenId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct MtWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_ids: Vec<nep245::TokenId>,
    pub amounts: Vec<U128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optionally make `storage_deposit` for `receiver_id` on `token`.
    /// The amount will be subtracted from user's NEP-141 `wNEAR` balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_deposit: Option<NearToken>,
}
