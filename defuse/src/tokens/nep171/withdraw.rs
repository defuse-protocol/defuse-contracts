use defuse_contracts::{
    defuse::{
        intents::tokens::NftWithdraw,
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
use near_contract_standards::{non_fungible_token, storage_management::ext_storage_management};
use near_plugins::{pause, Pausable};
use near_sdk::{
    assert_one_yocto, env, near, require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{
    accounts::Account, state::State, tokens::storage_management::STORAGE_DEPOSIT_GAS, DefuseImpl,
    DefuseImplExt,
};

const NFT_TRANSFER_GAS: Gas = Gas::from_tgas(15);

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
        storage_deposit: Option<NearToken>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = PREDECESSOR_ACCOUNT_ID.clone();
        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .nft_withdraw(
                sender_id,
                sender,
                NftWithdraw {
                    token,
                    receiver_id,
                    token_id,
                    memo,
                    storage_deposit,
                },
            )
            .unwrap_or_panic()
    }
}

impl State {
    pub fn nft_withdraw(
        &mut self,
        sender_id: AccountId,
        sender: &mut Account,
        withdraw @ NftWithdraw {
            storage_deposit, ..
        }: NftWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        if let Some(storage_deposit) = storage_deposit {
            Ok(self
                .unwrap_wnear(
                    sender_id.clone(),
                    sender,
                    storage_deposit,
                    Some("storage_deposit"),
                )?
                .then(
                    DefuseImpl::ext(CURRENT_ACCOUNT_ID.clone())
                        .with_static_gas(DefuseImpl::DO_NFT_WITHDRAW_GAS)
                        .do_nft_withdraw(sender_id, withdraw),
                )
                .into())
        } else {
            self.do_nft_withdraw(sender_id, sender, withdraw)
        }
    }
}

#[near]
impl DefuseImpl {
    const NFT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);
    const DO_NFT_WITHDRAW_GAS: Gas = Gas::from_tgas(5)
        // do_nft_withdraw() method is called only with storage_deposit
        .saturating_add(STORAGE_DEPOSIT_GAS)
        .saturating_add(NFT_TRANSFER_GAS)
        .saturating_add(Self::NFT_RESOLVE_WITHDRAW_GAS);

    #[private]
    pub fn do_nft_withdraw(
        &mut self,
        sender_id: AccountId,
        withdraw: NftWithdraw,
    ) -> PromiseOrValue<bool> {
        if withdraw.storage_deposit.is_some() {
            require!(
                matches!(env::promise_result(0), PromiseResult::Successful(data) if data == b"true"),
                "failed to unwrap wNEAR",
            );
        }

        let sender = self
            .accounts
            .get_mut(&sender_id)
            .ok_or(DefuseError::AccountNotFound)
            .unwrap_or_panic();
        self.state
            .do_nft_withdraw(sender_id, sender, withdraw)
            .unwrap_or_panic()
    }
}

impl State {
    fn do_nft_withdraw(
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
            [(TokenId::Nep171(token.clone(), token_id.clone()), 1)],
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = storage_deposit {
            ext_storage_management::ext(token.clone())
                .with_attached_deposit(storage_deposit)
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .storage_deposit(Some(receiver_id.clone()), None)
        } else {
            Promise::new(token.clone())
        }
        .nft_transfer(&receiver_id, &token_id, memo.as_deref())
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

pub trait NftExt {
    fn nft_transfer(
        self,
        receiver_id: &AccountId,
        token_id: &non_fungible_token::TokenId,
        memo: Option<&str>,
    ) -> Self;
}

impl NftExt for Promise {
    fn nft_transfer(
        self,
        receiver_id: &AccountId,
        token_id: &non_fungible_token::TokenId,
        memo: Option<&str>,
    ) -> Self {
        self.function_call(
            "nft_transfer".to_string(),
            serde_json::to_vec(&json!({
                "receiver_id": receiver_id,
                "token_id": token_id,
                "memo": memo,
            }))
            .unwrap_or_panic_display(),
            NearToken::from_yoctonear(1),
            NFT_TRANSFER_GAS,
        )
    }
}
