use defuse_contracts::{
    defuse::{tokens::TokenId, Result},
    utils::cache::CURRENT_ACCOUNT_ID,
    wnear::{ext_wnear, NEAR_WITHDRAW_GAS},
};
use near_sdk::{json_types::U128, AccountId, Gas, NearToken, Promise};

use crate::{accounts::Account, state::State, DefuseImpl};

pub const STORAGE_DEPOSIT_GAS: Gas = Gas::from_tgas(10);

impl State {
    pub fn unwrap_wnear(
        &mut self,
        account_id: AccountId,
        account: &mut Account,
        amount: NearToken,
        memo: Option<&str>,
    ) -> Result<Promise> {
        self.internal_withdraw(
            &account_id,
            account,
            [(
                TokenId::Nep141(self.wnear_id.clone()),
                amount.as_yoctonear(),
            )],
            memo,
        )?;

        let amount = U128(amount.as_yoctonear());

        Ok(ext_wnear::ext(self.wnear_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(NEAR_WITHDRAW_GAS)
            .near_withdraw(amount)
            .then(
                DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                    .with_static_gas(DefuseImpl::FT_RESOLVE_WITHDRAW_GAS)
                    .ft_resolve_withdraw(self.wnear_id.clone(), account_id, amount),
            ))
    }
}
