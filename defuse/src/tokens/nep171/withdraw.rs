use core::iter;

use defuse_contracts::{
    defuse::{
        intents::tokens::{NftWithdraw, StorageDeposit},
        tokens::{
            nep171::{NonFungibleTokenWithdrawResolver, NonFungibleTokenWithdrawer},
            TokenId,
        },
        DefuseError, Result,
    },
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        UnwrapOrPanic, UnwrapOrPanicError,
    },
};
use near_contract_standards::non_fungible_token;
use near_plugins::{pause, Pausable};
use near_sdk::{
    assert_one_yocto, env, near,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{accounts::Account, state::State, DefuseImpl, DefuseImplExt};

const NFT_TRANSFER_GAS: Gas = Gas::from_tgas(10);

#[near]
impl NonFungibleTokenWithdrawer for DefuseImpl {
    #[pause]
    #[payable]
    fn nft_withdraw(
        &mut self,
        token: AccountId,
        receiver_id: AccountId,
        token_id: non_fungible_token::TokenId,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        self.internal_nft_withdraw(
            PREDECESSOR_ACCOUNT_ID.clone(),
            NftWithdraw {
                token,
                receiver_id,
                token_id,
                memo,
                storage_deposit: None,
            },
        )
        .unwrap_or_panic()
    }
}

impl DefuseImpl {
    /// Value is taken from [`near_contract_standards`](https://github.com/near/near-sdk-rs/blob/f179a289528fbec5cd85077314e29deec198d0f3/near-contract-standards/src/non_fungible_token/core/core_impl.rs#L19)
    const NFT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);

    // TODO: export as #[private] for a backup?
    fn internal_nft_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: NftWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)?;
        self.state.nft_withdraw(sender_id, sender, withdraw)
    }
}

impl State {
    pub fn nft_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        NftWithdraw {
            token,
            receiver_id,
            token_id,
            memo,
            storage_deposit,
        }: NftWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        self.internal_withdraw(
            &sender_id,
            sender,
            iter::once((TokenId::Nep171(token.clone(), token_id.clone()), 1)).chain(
                storage_deposit.map(|amount| {
                    (
                        TokenId::Nep141(self.wnear_id.clone()),
                        amount.as_yoctonear(),
                    )
                }),
            ),
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
            "nft_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": &receiver_id,
                "token_id": &token_id,
                "memo": &memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            NFT_TRANSFER_GAS,
        )
        .then(
            DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(DefuseImpl::NFT_RESOLVE_WITHDRAW_GAS)
                .nft_resolve_withdraw(token, sender_id, token_id),
        )
        .into())
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
    ) -> bool {
        let ok =
            matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty());

        if !ok {
            self.internal_deposit(
                sender_id,
                [(TokenId::Nep171(token, token_id), 1)],
                Some("refund"),
            )
            .unwrap_or_panic();
        }
        ok
    }
}
