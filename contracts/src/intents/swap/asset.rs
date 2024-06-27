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

const GAS_FOR_NATIVE_TRANSFER: Gas = Gas::from_ggas(450);
// TODO: more accurate numbers
pub const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(20);
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas::from_tgas(20);

impl Asset {
    #[must_use]
    pub const fn gas_for_transfer(&self) -> Gas {
        match self {
            Self::Native(_) => GAS_FOR_NATIVE_TRANSFER,
            Self::Ft(_) => GAS_FOR_FT_TRANSFER,
            Self::Nft(_) => GAS_FOR_NFT_TRANSFER,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[serde_as]
#[near(serializers = [json, borsh])]
pub struct FtAmount {
    pub token: AccountId,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct NftItem {
    pub collection: AccountId,
    pub token_id: TokenId,
}
