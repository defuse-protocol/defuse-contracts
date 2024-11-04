use defuse_contracts::{
    defuse::{
        intents::tokens::{MtBatchTransfer, MtBatchTransferCall},
        DefuseError, Result,
    },
    nep245::{self, receiver::ext_mt_receiver, MultiTokenCore},
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_plugins::{pause, Pausable};
use near_sdk::{assert_one_yocto, json_types::U128, near, require, AccountId, Gas, PromiseOrValue};

use crate::{accounts::Account, intents::IntentExecutor, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenCore for DefuseImpl {
    #[payable]
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
    ) {
        self.mt_batch_transfer(
            receiver_id,
            [token_id].into(),
            [amount].into(),
            approval.map(|a| vec![Some(a)]),
            memo,
        );
    }

    #[pause(name = "mt_transfer")]
    #[payable]
    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        require!(approvals.is_none(), "approvals are not supported");

        self.internal_mt_batch_transfer(
            &PREDECESSOR_ACCOUNT_ID,
            MtBatchTransfer {
                receiver_id,
                token_ids: token_ids
                    .into_iter()
                    .map(|token_id| token_id.parse().unwrap_or_panic_display())
                    .collect(),
                amounts,
                memo,
            },
        )
        .unwrap_or_panic()
    }

    #[pause(name = "mt_transfer")]
    #[payable]
    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: nep245::TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.mt_batch_transfer_call(
            receiver_id,
            [token_id].into(),
            [amount].into(),
            approval.map(|a| vec![Some(a)]),
            memo,
            msg,
        )
    }

    #[pause(name = "mt_transfer")]
    #[payable]
    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();
        require!(approvals.is_none(), "approvals are not supported");

        self.internal_mt_batch_transfer_call(
            &PREDECESSOR_ACCOUNT_ID,
            MtBatchTransferCall {
                transfer: MtBatchTransfer {
                    receiver_id,
                    token_ids: token_ids
                        .into_iter()
                        .map(|token_id| token_id.parse().unwrap_or_panic_display())
                        .collect(),
                    amounts,
                    memo,
                },
                msg,
            },
            None,
        )
        .unwrap_or_panic()
    }

    fn mt_token(&self, token_ids: Vec<nep245::TokenId>) -> Vec<Option<nep245::Token>> {
        token_ids
            .into_iter()
            .map(|token_id| {
                self.total_supplies
                    .contains(&token_id.parse().ok()?)
                    .then_some(nep245::Token {
                        token_id,
                        owner_id: None,
                    })
            })
            .collect()
    }

    fn mt_balance_of(&self, account_id: AccountId, token_id: nep245::TokenId) -> U128 {
        U128(self.internal_mt_balance_of(&account_id, &token_id))
    }

    fn mt_batch_balance_of(
        &self,
        account_id: AccountId,
        token_ids: Vec<nep245::TokenId>,
    ) -> Vec<U128> {
        token_ids
            .into_iter()
            .map(|token_id| self.internal_mt_balance_of(&account_id, &token_id))
            .map(U128)
            .collect()
    }

    fn mt_supply(&self, token_id: nep245::TokenId) -> Option<U128> {
        Some(U128(
            self.total_supplies.balance_of(&token_id.parse().ok()?),
        ))
    }

    fn mt_batch_supply(&self, token_ids: Vec<nep245::TokenId>) -> Vec<Option<U128>> {
        token_ids
            .into_iter()
            .map(|token_id| self.mt_supply(token_id))
            .collect()
    }
}

impl DefuseImpl {
    fn internal_mt_batch_transfer(
        &mut self,
        sender_id: &AccountId,
        transfer: MtBatchTransfer,
    ) -> Result<()> {
        let sender = self
            .accounts
            .get_mut(sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state.execute_intent(sender_id, sender, transfer)
    }

    fn internal_mt_batch_transfer_call(
        &mut self,
        sender_id: &AccountId,
        transfer: MtBatchTransferCall,
        gas_for_mt_on_transfer: impl Into<Option<Gas>>,
    ) -> Result<PromiseOrValue<Vec<U128>>> {
        let sender = self
            .accounts
            .get_mut(sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state.internal_mt_batch_transfer_call(
            sender_id,
            sender,
            transfer,
            gas_for_mt_on_transfer,
        )
    }

    fn internal_mt_balance_of(&self, account_id: &AccountId, token_id: &nep245::TokenId) -> u128 {
        let Ok(token_id) = token_id.parse() else {
            return 0;
        };
        self.internal_balance_of(account_id, &token_id)
    }
}

impl State {
    pub fn internal_mt_batch_transfer_call(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        MtBatchTransferCall { transfer, msg }: MtBatchTransferCall,
        gas_for_mt_on_transfer: impl Into<Option<Gas>>,
    ) -> Result<PromiseOrValue<Vec<U128>>> {
        self.execute_intent(sender_id, sender, transfer.clone())?;

        let token_ids: Vec<_> = transfer.token_ids.iter().map(ToString::to_string).collect();

        let previous_owner_ids = vec![sender_id.clone(); token_ids.len()];

        let mut ext = ext_mt_receiver::ext(transfer.receiver_id.clone());
        if let Some(gas) = gas_for_mt_on_transfer.into() {
            ext = ext.with_static_gas(gas);
        }
        Ok(ext
            .mt_on_transfer(
                sender_id.clone(),
                previous_owner_ids.clone(),
                token_ids.clone(),
                transfer.amounts.clone(),
                msg,
            )
            .then(
                DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                    .with_static_gas(DefuseImpl::mt_resolve_transfer_gas(token_ids.len()))
                    .mt_resolve_transfer(
                        previous_owner_ids,
                        transfer.receiver_id,
                        token_ids,
                        transfer.amounts,
                        None,
                    ),
            )
            .into())
    }
}
