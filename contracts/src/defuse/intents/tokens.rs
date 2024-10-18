use impl_tools::autoimpl;
use near_contract_standards::non_fungible_token;
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, Gas};

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

    /// Optional static gas to attach to `mt_on_transfer`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_for_mt_on_transfer: Option<Gas>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct FtWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub amount: U128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Message to pass to `ft_transfer_call`. Otherwise, `ft_transfer` will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,

    /// Optional static gas to attach to `ft_transfer` or `ft_transfer_call`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas: Option<Gas>,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct NftWithdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id: non_fungible_token::TokenId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Message to pass to `nft_transfer_call`. Otherwise, `nft_transfer` will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,

    /// Optional static gas to attach to `nft_transfer` or `nft_transfer_call`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas: Option<Gas>,
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

    /// Message to pass to `mt_batch_transfer_call`. Otherwise, `mt_batch_transfer` will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,

    /// Optional static gas to attach to `mt_batch_transfer` or `mt_batch_transfer_call`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas: Option<Gas>,
}
