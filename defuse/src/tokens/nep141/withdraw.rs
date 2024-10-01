use defuse_contracts::{
    defuse::{
        tokens::{
            nep141::{FungibleTokenWithdrawResolver, FungibleTokenWithdrawer},
            TokenId,
        },
        Result,
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic,
    },
};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, require, serde_json, AccountId, Gas, NearToken,
    PromiseOrValue, PromiseResult,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl FungibleTokenWithdrawer for DefuseImpl {
    #[payable]
    fn ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        self.internal_ft_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            receiver_id,
            token,
            amount,
            memo,
            msg,
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    /// Value is taken from [`near_contract_standards`](https://github.com/near/near-sdk-rs/blob/f179a289528fbec5cd85077314e29deec198d0f3/near-contract-standards/src/fungible_token/core_impl.rs#L12)
    const FT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);

    fn internal_ft_withdraw(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        token: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> Result<PromiseOrValue<U128>> {
        require!(amount.0 > 0, "zero amount");

        self.internal_withdraw(&sender_id, [(TokenId::Nep141(token.clone()), amount.0)])?;

        let ext =
            ext_ft_core::ext(token.clone()).with_attached_deposit(NearToken::from_yoctonear(1));
        let is_call = msg.is_some();
        Ok(if let Some(msg) = msg {
            ext.ft_transfer_call(receiver_id, amount, memo, msg)
        } else {
            ext.ft_transfer(receiver_id, amount, memo)
        }
        .then(
            Self::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(Self::FT_RESOLVE_WITHDRAW_GAS)
                .ft_resolve_withdraw(token, sender_id, amount, is_call),
        )
        .into())
    }
}

#[near]
impl FungibleTokenWithdrawResolver for DefuseImpl {
    #[private]
    fn ft_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        amount: U128,
        is_call: bool,
    ) -> U128 {
        let used = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if is_call {
                    // `ft_transfer_call` returns successfully transferred amount
                    serde_json::from_slice::<U128>(&value).unwrap_or_default().0
                } else if value.is_empty() {
                    // `ft_transfer` returns empty result on success
                    amount.0
                } else {
                    0
                }
            }
            PromiseResult::Failed => 0,
        }
        .min(amount.0);

        let refund = amount.0 - used;
        if refund > 0 {
            let token = TokenId::Nep141(token);
            self.total_supplies
                .deposit(token.clone(), refund)
                .unwrap_or_panic();
            self.accounts
                .get_or_create(sender_id)
                .token_balances
                .deposit(token, refund)
                .unwrap_or_panic();
        }
        U128(used)
    }
}
