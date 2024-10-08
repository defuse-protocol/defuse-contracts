use impl_tools::autoimpl;
use near_contract_standards::non_fungible_token;
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, Gas};

use crate::nep245;

use crate::defuse::tokens::TokenAmounts;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub receiver_id: AccountId,
    pub token_id_amounts: TokenAmounts<u128>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[near(serializers = [borsh, json])]
#[autoimpl(Deref using self.transfer)]
#[derive(Debug, Clone)]
pub struct TokenTransferCall {
    #[serde(flatten)]
    pub transfer: TokenTransfer,

    /// `msg` to pass in `mt_on_transfer`
    pub msg: String,

    /// Optional static gas to attach to `mt_on_transfer`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_for_mt_on_transfer: Option<Gas>,
}

#[near(serializers = [borsh, json])]
#[serde(tag = "token_standard", rename_all = "snake_case")]
#[derive(Debug, Clone)]
pub enum TokenWithdraw {
    Nep141(Nep141Withdraw),
    Nep171(Nep171Withdraw),
    Nep245(Nep245Withdraw),
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Nep141Withdraw {
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
pub struct Nep171Withdraw {
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
pub struct Nep245Withdraw {
    pub token: AccountId,
    pub receiver_id: AccountId,
    pub token_id_amounts: Vec<(nep245::TokenId, U128)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Message to pass to `mt_batch_transfer_call`. Otherwise, `mt_batch_transfer` will be used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,

    /// Optional static gas to attach to `mt_batch_transfer` or `mt_batch_transfer_call`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas: Option<Gas>,
}
