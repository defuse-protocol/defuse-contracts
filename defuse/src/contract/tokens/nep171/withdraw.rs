use core::iter;

use defuse_core::{intents::tokens::NftWithdraw, tokens::TokenId, Result};
use defuse_near_utils::{
    UnwrapOrPanic, UnwrapOrPanicError, CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID,
};
use defuse_wnear::{ext_wnear, NEAR_WITHDRAW_GAS};
use near_contract_standards::{non_fungible_token, storage_management::ext_storage_management};
use near_plugins::{access_control_any, pause, AccessControllable, Pausable};
use near_sdk::{
    assert_one_yocto, env,
    json_types::U128,
    near, require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise, PromiseOrValue, PromiseResult,
};

use crate::{
    contract::{tokens::STORAGE_DEPOSIT_GAS, Contract, ContractExt, Role},
    tokens::nep171::{
        NonFungibleTokenForceWithdrawer, NonFungibleTokenWithdrawResolver,
        NonFungibleTokenWithdrawer,
    },
};

const NFT_TRANSFER_GAS: Gas = Gas::from_tgas(15);

#[near]
impl NonFungibleTokenWithdrawer for Contract {
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

impl Contract {
    pub(crate) fn internal_nft_withdraw(
        &mut self,
        owner_id: AccountId,
        withdraw: NftWithdraw,
    ) -> Result<PromiseOrValue<bool>> {
        self.internal_withdraw(
            &owner_id,
            iter::once((
                TokenId::Nep171(withdraw.token.clone(), withdraw.token_id.clone()),
                1,
            ))
            .chain(withdraw.storage_deposit.map(|amount| {
                (
                    TokenId::Nep141(self.wnear_id.clone()),
                    amount.as_yoctonear(),
                )
            })),
            Some("withdraw"),
        )?;

        Ok(if let Some(storage_deposit) = withdraw.storage_deposit {
            ext_wnear::ext(self.wnear_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(NEAR_WITHDRAW_GAS)
                .near_withdraw(U128(storage_deposit.as_yoctonear()))
                .then(
                    // schedule storage_deposit() only after near_withdraw() returns
                    Contract::ext(CURRENT_ACCOUNT_ID.clone())
                        .with_static_gas(Contract::DO_NFT_WITHDRAW_GAS)
                        .do_nft_withdraw(withdraw.clone()),
                )
        } else {
            Contract::do_nft_withdraw(withdraw.clone())
        }
        .then(
            Contract::ext(CURRENT_ACCOUNT_ID.clone())
                .with_static_gas(Contract::NFT_RESOLVE_WITHDRAW_GAS)
                .nft_resolve_withdraw(withdraw.token, owner_id, withdraw.token_id),
        )
        .into())
    }
}

#[near]
impl Contract {
    const NFT_RESOLVE_WITHDRAW_GAS: Gas = Gas::from_tgas(5);
    const DO_NFT_WITHDRAW_GAS: Gas = Gas::from_tgas(3)
        // do_nft_withdraw() method is called externally
        // only with storage_deposit
        .saturating_add(STORAGE_DEPOSIT_GAS)
        .saturating_add(NFT_TRANSFER_GAS);

    #[private]
    pub fn do_nft_withdraw(withdraw: NftWithdraw) -> Promise {
        if let Some(storage_deposit) = withdraw.storage_deposit {
            require!(
                matches!(env::promise_result(0), PromiseResult::Successful(data) if data.is_empty()),
                "near_withdraw failed",
            );

            ext_storage_management::ext(withdraw.token)
                .with_attached_deposit(storage_deposit)
                .with_static_gas(STORAGE_DEPOSIT_GAS)
                .storage_deposit(Some(withdraw.receiver_id.clone()), None)
        } else {
            Promise::new(withdraw.token)
        }
        .nft_transfer(
            &withdraw.receiver_id,
            &withdraw.token_id,
            withdraw.memo.as_deref(),
        )
    }
}

#[near]
impl NonFungibleTokenWithdrawResolver for Contract {
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

#[near]
impl NonFungibleTokenForceWithdrawer for Contract {
    #[access_control_any(roles(Role::DAO, Role::UnrestrictedWithdrawer))]
    #[payable]
    fn nft_force_withdraw(
        &mut self,
        owner_id: AccountId,
        token: AccountId,
        receiver_id: AccountId,
        token_id: non_fungible_token::TokenId,
        memo: Option<String>,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        self.internal_nft_withdraw(
            owner_id,
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