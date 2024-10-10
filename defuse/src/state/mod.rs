pub mod runtime;

use defuse_contracts::utils::{fees::Pips, prefix::NestPrefix};
use impl_tools::autoimpl;
use near_sdk::{borsh::BorshSerialize, near, AccountId, BorshStorageKey, IntoStorageKey};

use crate::tokens::TokensBalances;

use self::runtime::RuntimeState;

#[near(serializers = [borsh])]
#[autoimpl(Deref using self.runtime)]
#[autoimpl(DerefMut using self.runtime)]
#[derive(Debug)]
pub struct State {
    pub fee: Pips,
    pub fee_collector: AccountId,
    pub total_supplies: TokensBalances,

    #[borsh(skip)]
    pub runtime: RuntimeState,
}

impl State {
    #[inline]
    pub fn new<S>(prefix: S, fee: Pips, fee_collector: AccountId) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            fee,
            fee_collector,
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
