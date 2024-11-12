use defuse_contracts::{
    defuse::{intents::tokens::NativeWithdraw, tokens::TokenId, Result},
    utils::cache::CURRENT_ACCOUNT_ID,
    wnear::{ext_wnear, NEAR_WITHDRAW_GAS},
};
use near_sdk::{
    env, json_types::U128, near, require, AccountId, Gas, NearToken, Promise, PromiseResult,
};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

impl State {
    pub fn native_withdraw(
        &mut self,
        owner_id: &AccountId,
        owner: &mut Account,
        withdraw: NativeWithdraw,
    ) -> Result<Promise> {
        self.internal_withdraw(
            owner_id,
            owner,
            [(
                TokenId::Nep141(self.wnear_id.clone()),
                withdraw.amount.as_yoctonear(),
            )],
            Some("withdraw"),
        )?;

        Ok(ext_wnear::ext(self.wnear_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(NEAR_WITHDRAW_GAS)
            .near_withdraw(U128(withdraw.amount.as_yoctonear()))
            .then(
                // do_native_withdraw only after unwrapping NEAR
                DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                    .with_static_gas(DefuseImpl::DO_NATIVE_WITHDRAW_GAS)
                    .do_native_withdraw(withdraw),
            ))
    }
}

#[near]
impl DefuseImpl {
    const DO_NATIVE_WITHDRAW_GAS: Gas = Gas::from_tgas(3)
        // Transfer NEAR
        .saturating_add(Gas::from_tgas(1));

    #[private]
    pub fn do_native_withdraw(&mut self, withdraw: NativeWithdraw) -> Promise {
        require!(
            matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty()),
            "near_withdraw failed",
        );

        Promise::new(withdraw.receiver_id).transfer(withdraw.amount)
    }
}
