use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::{core::NonFungibleTokenReceiver, TokenId},
};
use near_sdk::{env, ext_contract, near, AccountId, Gas, NearToken, Promise, PromiseOrValue};
use serde_with::{serde_as, DefaultOnNull, DisplayFromStr};

pub use self::error::*;

mod error;

pub type IntentID = String;

#[ext_contract(ext_swap_intent)]
pub trait SwapIntentContract: FungibleTokenReceiver + NonFungibleTokenReceiver {
    fn get_swap_intent(&self, id: &IntentID) -> Option<&SwapIntent>;

    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<()>;

    fn rollback_intent(&mut self, id: IntentID) -> Promise;
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case")]
pub enum SwapIntentAction {
    Create(CreateSwapIntentAction),
    Fulfill(FulfillSwapIntentAction),
}

#[derive(Debug, Clone)]
#[serde_as]
#[near(serializers = [json, borsh])]
pub struct CreateSwapIntentAction {
    /// This should not exist before
    pub id: IntentID,
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
#[near(serializers = [json, borsh])]
pub struct FulfillSwapIntentAction {
    pub id: IntentID,
    /// By default: back to sender
    pub recipient: Option<AccountId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
#[serde(rename_all = "snake_case")]
pub enum Asset {
    /// NEAR
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

#[derive(Debug, Clone)]
#[serde_as]
#[near(serializers = [borsh, json])]
pub struct SwapIntent {
    pub initiator: AccountId,
    pub asset_in: Asset,
    // TODO: in case of NFT, this only allows for simple "barter",
    // while in case of Defuse, the user doesn't know in advance which
    // account solver will use for this swap. Possible solutions for this issue:
    // * Accept whatever NFT from only whitelisted solvers
    // * Some kind of auction, where solvers "register" their willingness
    //   to close the intent and compete between each other over given
    //   set of preperties. These properties of suggested addresses by solvers
    //   can be compared between each other either on-chain (by having
    //   light-client contracts for each chain) or by user front-ends:
    //   this info about offers can be presented to the user and user can
    //   accept the best one or chose between them.
    //   So,it will become 3-stage process. We need to thing about it properly
    pub asset_out: Asset,
    /// By default, sender
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull")]
    pub recipient: Option<AccountId>,
    // TODO: prolong() method
    pub deadline: Deadline,
}

impl SwapIntent {
    #[inline]
    pub fn has_expired(&self) -> bool {
        self.deadline.has_expired()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers=[borsh, json])]
#[serde(rename_all = "snake_case")]
pub enum Deadline {
    /// UNIX Timestamp in seconds
    Timestamp(u64),
    /// Block number
    BlockNumber(u64),
}

impl Deadline {
    #[inline]
    pub fn has_expired(self) -> bool {
        match self {
            Self::Timestamp(timestamp) => {
                env::block_timestamp_ms() > timestamp.saturating_mul(1000)
            }
            Self::BlockNumber(n) => env::block_height() > n,
        }
    }
}
