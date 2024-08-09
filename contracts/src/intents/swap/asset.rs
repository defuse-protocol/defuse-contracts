use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{json_types::U128, near, AccountId, Gas, NearToken};

use super::CrossChainAsset;

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(untagged)]
pub enum AssetWithAccount {
    Near {
        #[serde(flatten)]
        asset: NearAsset,
        /// Sender in context of `asset_in`, recipient for `asset_out`
        account: AccountId,
    },
    CrossChain {
        #[serde(flatten)]
        asset: CrossChainAsset,
        /// Sender in context of `asset_in`, recipient for `asset_out`
        account: String,
    },
}

impl AssetWithAccount {
    #[must_use]
    #[inline]
    pub fn new(asset: Asset, account: GenericAccount) -> Option<Self> {
        Some(match (asset, account) {
            (Asset::Near(asset), GenericAccount::Near(account)) => Self::Near { account, asset },
            (Asset::CrossChain(asset), GenericAccount::CrossChain(account)) => {
                Self::CrossChain { account, asset }
            }
            _ => return None,
        })
    }

    #[must_use]
    #[inline]
    pub const fn initiator(&self) -> &AccountId {
        match self {
            Self::Near { account, .. } => account,
            Self::CrossChain {
                asset: CrossChainAsset { oracle, .. },
                ..
            } => oracle,
        }
    }

    #[must_use]
    #[inline]
    pub fn asset(&self) -> Asset {
        match self {
            Self::Near { asset, .. } => Asset::Near(asset.clone()),
            Self::CrossChain { asset, .. } => Asset::CrossChain(asset.clone()),
        }
    }

    #[must_use]
    #[inline]
    pub fn account(&self) -> GenericAccount {
        match self {
            Self::Near { account, .. } => GenericAccount::Near(account.clone()),
            Self::CrossChain { account, .. } => GenericAccount::CrossChain(account.clone()),
        }
    }

    #[must_use]
    #[inline]
    pub fn with_account(&self, account: GenericAccount) -> Option<Self> {
        Self::new(self.asset(), account)
    }

    #[must_use]
    #[inline]
    pub const fn gas_for_refund(&self) -> Gas {
        match self {
            Self::Near { asset, .. } => asset.gas_for_refund(),
            Self::CrossChain { asset, .. } => asset.gas_for_refund(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_zero_amount(&self) -> bool {
        match self {
            Self::Near { asset, .. } => asset.is_zero_amount(),
            Self::CrossChain { asset, .. } => asset.is_zero_amount(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum NearAsset {
    /// Native NEAR
    Native { amount: NearToken },
    /// NEP-141
    Nep141(FtAmount),
    /// NEP-171
    Nep171(NftItem),
}

impl NearAsset {
    const GAS_FOR_NATIVE_TRANSFER: Gas = Gas::from_ggas(450);
    // TODO: more accurate numbers
    pub const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(20);
    pub const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(20);

    #[must_use]
    #[inline]
    pub const fn gas_for_transfer(&self) -> Gas {
        match self {
            Self::Native { .. } => Self::GAS_FOR_NATIVE_TRANSFER,
            Self::Nep141(_) => Self::GAS_FOR_FT_TRANSFER,
            Self::Nep171(_) => Self::GAS_FOR_NFT_TRANSFER,
        }
    }

    #[must_use]
    #[inline]
    pub const fn gas_for_refund(&self) -> Gas {
        match self {
            // native asset can only be refunded manually
            Self::Native { .. } => Self::GAS_FOR_NATIVE_TRANSFER,
            // other assets are refunded within *_resolve_transfer()
            _ => Gas::from_gas(0),
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_zero_amount(&self) -> bool {
        match self {
            Self::Native { amount } => amount.is_zero(),
            Self::Nep141(FtAmount { amount, .. }) => amount.0 == 0,
            Self::Nep171(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asset {
    Near(NearAsset),
    CrossChain(CrossChainAsset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct FtAmount {
    /// Token account
    pub token: AccountId,
    /// Amount of tokens
    pub amount: U128,
}

impl FtAmount {
    #[must_use]
    pub const fn new(token_id: AccountId, amount: u128) -> Self {
        Self {
            token: token_id,
            amount: U128(amount),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct NftItem {
    /// Collection account
    pub collection: AccountId,
    /// Token ID
    pub token_id: TokenId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case", tag = "type", content = "address")]
pub enum GenericAccount {
    Near(AccountId),
    CrossChain(String),
}

impl From<AccountId> for GenericAccount {
    fn from(value: AccountId) -> Self {
        Self::Near(value)
    }
}

impl From<String> for GenericAccount {
    fn from(value: String) -> Self {
        Self::CrossChain(value)
    }
}
