pub mod runtime;

use defuse_contracts::utils::prefix::NestPrefix;
use impl_tools::autoimpl;
use near_sdk::{borsh::BorshSerialize, near, AccountId, BorshStorageKey, IntoStorageKey};

use crate::{fees::FeesConfig, tokens::TokensBalances};

use self::runtime::RuntimeState;

#[near(serializers = [borsh])]
#[autoimpl(Deref using self.runtime)]
#[autoimpl(DerefMut using self.runtime)]
#[derive(Debug)]
pub struct State {
    pub total_supplies: TokensBalances,

    pub wnear_id: AccountId,

    pub fees: FeesConfig,

    #[borsh(skip)]
    pub runtime: RuntimeState,
}

impl State {
    #[inline]
    pub fn new<S>(prefix: S, wnear_id: AccountId, fees: FeesConfig) -> Self
    where
        S: IntoStorageKey,
    {
        Self {
            total_supplies: TokensBalances::new(
                prefix.into_storage_key().nest(Prefix::TotalSupplies),
            ),
            wnear_id,
            fees,
            runtime: Default::default(),
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum Prefix {
    TotalSupplies,
}
