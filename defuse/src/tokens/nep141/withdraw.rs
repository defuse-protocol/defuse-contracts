use core::iter;

use defuse_contracts::{
    defuse::{
        intents::tokens::{FtWithdraw, StorageDeposit},
        tokens::{
            nep141::{FungibleTokenWithdrawResolver, FungibleTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_plugins::{pause, Pausable};
use near_sdk::{
    assert_one_yocto, env,
    json_types::U128,
    near,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

const FT_TRANSFER_GAS: Gas = Gas::from_tgas(10);

#[near]
impl FungibleTokenWithdrawer for DefuseImpl {
    #[pause]
    #[payable]
    fn ft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        self.internal_ft_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            FtWithdraw {
                token,
                receiver_id,
                amount,
                memo,
                storage_deposit: None,
            },
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    /// Value is taken from [`near_contract_standards`](https://github.com/near/near-sdk-rs/blob/f179a289528fbec5cd85077314e29deec198d0f3/near-contract-standards/src/fungible_token/core_impl.rs#L12)
    const FT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);

    #[inline]
    // TODO: export as #[private] for a backup?
    fn internal_ft_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: FtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state.ft_withdraw(sender_id, sender, withdraw)
    }
}

impl State {
    pub fn ft_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        FtWithdraw {
            token,
            receiver_id,
            amount,
            memo,
            storage_deposit,
        }: FtWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        self.internal_withdraw(
            &sender_id,
            sender,
            iter::once((TokenId::Nep141(token.clone()), amount.0)).chain(storage_deposit.map(
                |amount| {
                    (
                        TokenId::Nep141(self.wnear_id.clone()),
                        amount.as_yoctonear(),
                    )
                },
            )),
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = storage_deposit {
            self.internal_storage_deposit(StorageDeposit {
                contract_id: token.clone(),
                account_id: receiver_id.clone(),
                amount: storage_deposit,
            })
        } else {
            Promise::new(token.clone())
        }
        .function_call(
            "ft_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": &receiver_id,
                "amount": &amount,
                "memo": &memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            FT_TRANSFER_GAS,
        )
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::FT_RESOLVE_WITHDRAW_GAS)
                .ft_resolve_withdraw(token, sender_id, amount),
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
    ) -> bool {
        let ok =
            matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty());

        if !ok {
            self.internal_deposit(
                sender_id,
                [(TokenId::Nep141(token), amount.0)],
                Some("refund"),
            )
            .unwrap_or_panic();
        }
        ok
    }
}
