use defuse_contracts::{
    defuse::{
        tokens::{
            nep141::{FungibleTokenWithdrawResolver, FungibleTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    utils::cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, serde_json, AccountId, NearToken,
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
        self.internal_ft_withdraw(token, receiver_id, amount, memo, msg)
            .unwrap()
    }
}

impl DefuseImpl {
    fn internal_ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: Option<String>,
    ) -> Result<PromiseOrValue<U128>> {
        // TODO: check amount > 0
        let token_id = TokenId::Nep141(token.clone());
        self.total_supplies.withdraw(&token_id, amount.0)?;
        self.accounts
            .get_mut(&PREDECESSOR_ACCOUNT_ID)
            .ok_or(DefuseError::AccountNotFound)?
            .token_balances
            .withdraw(&token_id, amount.0)?;

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
                // TODO: with static gas
                .ft_resolve_withdraw(token, PREDECESSOR_ACCOUNT_ID.clone(), amount, is_call),
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
            self.total_supplies.deposit(token.clone(), refund).unwrap();
            self.accounts
                .get_or_create(sender_id)
                .token_balances
                .deposit(token, refund)
                .unwrap();
        }
        U128(used)
    }
}
