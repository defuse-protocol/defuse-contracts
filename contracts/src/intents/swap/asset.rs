use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{near, AccountId, Gas, NearToken};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    /// Native NEAR
    Native(NearToken),
    /// NEP-141
    Ft(FtAmount),
    /// NEP-171
    Nft(NftItem),
}

impl Asset {
    pub const GAS_FOR_NATIVE_TRANSFER: Gas = Gas::from_ggas(450);
    // TODO: more accurate numbers
    pub const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(20);
    pub const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(20);

    #[must_use]
    pub const fn gas_for_transfer(&self) -> Gas {
        match self {
            Self::Native(_) => Self::GAS_FOR_NATIVE_TRANSFER,
            Self::Ft(_) => Self::GAS_FOR_FT_TRANSFER,
            Self::Nft(_) => Self::GAS_FOR_NFT_TRANSFER,
        }
    }
}

impl Asset {
    #[must_use]
    #[inline]
    pub const fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    #[must_use]
    #[inline]
    pub const fn is_ft(&self) -> bool {
        matches!(self, Self::Ft(_))
    }

    #[must_use]
    #[inline]
    pub const fn is_nft(&self) -> bool {
        matches!(self, Self::Nft(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[serde_as]
#[near(serializers = [json, borsh])]
pub struct FtAmount {
    /// Token account
    pub token: AccountId,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct NftItem {
    /// Collection account
    pub collection: AccountId,
    /// Token ID
    pub token_id: TokenId,
}
