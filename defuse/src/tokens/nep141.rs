use defuse_contracts::{
    defuse::{
        tokens::{nep141::FungibleTokenWithdrawer, DepositMessage, TokenId},
        DefuseError,
    },
    utils::cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
};
use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{
    assert_one_yocto, env, json_types::U128, near, serde_json, AccountId, NearToken, Promise,
    PromiseError, PromiseOrValue,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl FungibleTokenReceiver for DefuseImpl {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.internal_ft_on_transfer(sender_id, amount, &msg)
            .map(|()| PromiseOrValue::Value(U128(0)))
            .unwrap()
    }
}

#[near]
impl FungibleTokenWithdrawer for DefuseImpl {
    #[payable]
    fn nep141_withdraw(
        &mut self,
        token_id: AccountId,
        to: Option<AccountId>,
        amount: U128,
    ) -> Promise {
        assert_one_yocto();
        self.internal_withdraw_nep141(
            token_id,
            to.unwrap_or_else(env::predecessor_account_id),
            amount.0,
            None,
        )
        .unwrap()
    }

    #[payable]
    fn nep141_withdraw_call(
        &mut self,
        token_id: AccountId,
        to: Option<AccountId>,
        amount: U128,
        msg: String,
    ) -> Promise {
        assert_one_yocto();
        self.internal_withdraw_nep141(
            token_id,
            to.unwrap_or_else(env::predecessor_account_id),
            amount.0,
            Some(msg),
        )
        .unwrap()
    }
}

impl DefuseImpl {
    fn internal_ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: &str,
    ) -> Result<(), DefuseError> {
        let msg: DepositMessage = Some(msg)
            .filter(|msg| !msg.is_empty())
            .map(serde_json::from_str)
            .transpose()?
            .unwrap_or_default();

        let token = TokenId::Nep141(env::predecessor_account_id());
        let account = self
            .accounts
            .get_or_create(msg.deposit_to.unwrap_or(sender_id));
        account.token_balances.deposit(token, amount.0)?;

        for action in msg.actions {
            // TODO: pass predecessor_id?
            self.execute_action(action)?;
        }

        Ok(())
    }

    fn internal_withdraw_nep141(
        &mut self,
        token_id: AccountId,
        to: AccountId,
        amount: u128,
        msg: Option<String>,
    ) -> Result<Promise, DefuseError> {
        let account = self
            .accounts
            .get_mut(&PREDECESSOR_ACCOUNT_ID)
            .ok_or(DefuseError::AccountNotFound)?;
        account
            .token_balances
            .withdraw(&TokenId::Nep141(token_id.clone()), amount)?;

        Ok(if let Some(msg) = msg {
            ext_ft_core::ext(token_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer_call(
                    to.clone(),
                    U128(amount),
                    None, // TODO
                    msg,
                )
                .then(
                    Self::ext(CURRENT_ACCOUNT_ID.clone())
                        // TODO: static gas
                        .resolve_nep141_withdraw_call(
                            token_id,
                            PREDECESSOR_ACCOUNT_ID.clone(),
                            U128(amount),
                        ),
                )
        } else {
            ext_ft_core::ext(token_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer(
                    to.clone(),
                    U128(amount),
                    None, // TODO
                )
                .then(
                    Self::ext(CURRENT_ACCOUNT_ID.clone())
                        // TODO: static gas
                        .resolve_nep141_withdraw(
                            token_id,
                            PREDECESSOR_ACCOUNT_ID.clone(),
                            U128(amount),
                        ),
                )
        })
    }
}

#[near]
impl DefuseImpl {
    #[private]
    pub fn resolve_nep141_withdraw(
        &mut self,
        token_id: AccountId,
        from: AccountId,
        amount: U128,
        #[callback_result] ft_transfer_ok: Result<(), PromiseError>,
    ) -> bool {
        if ft_transfer_ok.is_err() {
            let account = self.accounts.get_or_create(from);
            // Are we sure that we want to ignore that?
            let _ = account
                .token_balances
                .deposit(TokenId::Nep141(token_id), amount.0);
        }
        ft_transfer_ok.is_ok()
    }

    #[private]
    pub fn resolve_nep141_withdraw_call(
        &mut self,
        token_id: AccountId,
        from: AccountId,
        amount: U128,
        #[callback_result] ft_transfer_call_result: Result<U128, PromiseError>,
    ) -> U128 {
        let used = ft_transfer_call_result.ok().unwrap_or(amount).min(amount);
        let refund = amount.0 - used.0;
        if refund > 0 {
            let account = self.accounts.get_or_create(from);
            // ignore refund error
            let _ = account
                .token_balances
                .deposit(TokenId::Nep141(token_id), refund);
        }
        used
    }
}
