use near_contract_standards::{
    fungible_token::receiver::FungibleTokenReceiver,
    non_fungible_token::{core::NonFungibleTokenReceiver, TokenId},
};
use near_sdk::{env, ext_contract, near, AccountId, Gas, NearToken, Promise, PromiseOrValue};

pub use self::error::*;

mod error;

pub type IntentID = String;

#[ext_contract(ext_swap_intent)]
pub trait SwapIntentContract: FungibleTokenReceiver + NonFungibleTokenReceiver {
    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<()>;

    fn rollback_intent(&mut self, id: IntentID) -> Promise;
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub enum SwapIntentAction {
    Create(CreateSwapIntentAction),
    Fulfill(FulfillSwapIntentAction),
}

#[derive(Debug, Clone)]
#[near(serializers = [json, borsh])]
pub struct CreateSwapIntentAction {
    /// This should not exist before
    pub id: IntentID,
    /// Desired asset as an output
    pub asset_out: Asset,
    /// Where to send asset_out.
    /// By default: back to sender
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
#[near(serializers = [json, borsh])]
pub struct FtAmount {
    pub token: AccountId,
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct NftItem {
    pub collection: AccountId,
    pub token_id: TokenId,
}

#[derive(Debug, Clone)]
#[near(serializers = [borsh])]
pub struct SwapIntent {
    pub sender: AccountId,
    pub asset_in: Asset,
    pub asset_out: Asset,
    /// By default, sender
    pub recipient: Option<AccountId>,
    /// The deadline for which
    /// This is intentionally not optional to forbid the user
    /// to lock his asset for potentionally eternity.
    // TODO: prolong() method
    pub deadline: Deadline,
    // TODO: expiration
}

impl SwapIntent {
    #[inline]
    pub fn has_expired(&self) -> bool {
        self.deadline.has_expired()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers=[borsh, json])]
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
