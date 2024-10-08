pub mod runtime;

use defuse_contracts::{defuse::fees::Fees, utils::prefix::NestPrefix};
use impl_tools::autoimpl;
use near_sdk::{borsh::BorshSerialize, near, BorshStorageKey, IntoStorageKey};

use crate::tokens::TokensBalances;

use self::runtime::RuntimeState;

#[near(serializers = [borsh])]
#[autoimpl(Deref using self.runtime)]
#[autoimpl(DerefMut using self.runtime)]
#[derive(Debug)]
pub struct State {
    pub fees: Fees,
    pub total_supplies: TokensBalances,

    #[borsh(skip)]
    pub runtime: RuntimeState,
}

impl State {
    #[inline]
    pub fn new<S>(prefix: S, fees: Fees) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            fees,
            total_supplies: TokensBalances::new(
                prefix.into_storage_key().nest(Prefix::TotalSupplies),
            ),
            runtime: Default::default(),
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum Prefix {
    TotalSupplies,
}
