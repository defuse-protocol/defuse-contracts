use defuse_contracts::{
    defuse::{
        intents::tokens::Nep171Withdraw,
        tokens::{
            nep171::{NonFungibleTokenWithdrawResolver, NonFungibleTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic,
    },
};
use near_contract_standards::non_fungible_token::{self, core::ext_nft_core};
use near_sdk::{
    assert_one_yocto, env, near, serde_json, AccountId, Gas, NearToken, Promise, PromiseOrValue,
    PromiseResult,
};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

#[near]
impl NonFungibleTokenWithdrawer for DefuseImpl {
    #[payable]
    fn nft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_id: non_fungible_token::TokenId,
        memo: Option<String>,
        msg: Option<String>,
        gas: Option<Gas>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        self.internal_nft_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            Nep171Withdraw {
                token,
                receiver_id,
                token_id,
                memo,
                msg,
                gas,
            },
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    /// Value is taken from [`near_contract_standards`](https://github.com/near/near-sdk-rs/blob/f179a289528fbec5cd85077314e29deec198d0f3/near-contract-standards/src/non_fungible_token/core/core_impl.rs#L19)
    const NFT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);

    fn internal_nft_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: Nep171Withdraw,
    ) -> Result<PromiseOrValue<bool>> {
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state
            .nft_withdraw(sender_id, sender, withdraw)
            .map(Into::into)
    }
}

impl State {
    pub fn nft_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        Nep171Withdraw {
            token,
            receiver_id,
            token_id,
            memo,
            msg,
            gas,
        }: Nep171Withdraw,
    ) -> Result<Promise> {
        self.internal_withdraw(
            sender,
            [(TokenId::Nep171(token.clone(), token_id.clone()), 1)],
        )?;

        let mut ext =
            ext_nft_core::ext(token.clone()).with_attached_deposit(NearToken::from_yoctonear(1));
        if let Some(gas) = gas {
            ext = ext.with_static_gas(gas);
        }
        let is_call = msg.is_some();
        Ok(if let Some(msg) = msg {
            ext.nft_transfer_call(receiver_id, token_id.clone(), None, memo, msg)
        } else {
            ext.nft_transfer(receiver_id, token_id.clone(), None, memo)
        }
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::NFT_RESOLVE_WITHDRAW_GAS)
                .nft_resolve_withdraw(token, sender_id, token_id, is_call),
        ))
    }
}

#[near]
impl NonFungibleTokenWithdrawResolver for DefuseImpl {
    #[private]
    fn nft_resolve_withdraw(
        &mut self,
        token: AccountId,
        sender_id: AccountId,
        token_id: non_fungible_token::TokenId,
        is_call: bool,
    ) -> bool {
        let used = match env::promise_result(0) {
            PromiseResult::Successful(value) => {
                if is_call {
                    // `nft_transfer_call` returns true if token was successfully transferred
                    serde_json::from_slice(&value).unwrap_or_default()
                } else {
                    // `nft_transfer` returns empty result on success
                    value.is_empty()
                }
            }
            PromiseResult::Failed => false,
        };
        if !used {
            let token = TokenId::Nep171(token, token_id);
            self.total_supplies
                .deposit(token.clone(), 1)
                .unwrap_or_panic();
            self.accounts
                .get_or_create(sender_id)
                .token_balances
                .deposit(token, 1)
                .unwrap_or_panic();
        }
        used
    }
}
