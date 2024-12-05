use std::borrow::Cow;

use defuse_core::{
    accounts::AccountEvent,
    crypto::PublicKey,
    engine::{Inspector, State, StateView},
    events::DefuseEvent,
    fees::Pips,
    intents::{
        token_diff::TokenDiff,
        tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
        IntentExecutedEvent,
    },
    tokens::TokenId,
    Deadline, DefuseError, Nonce, Result,
};
use defuse_near_utils::CURRENT_ACCOUNT_ID;
use defuse_nep245::{MtBurnEvent, MtEvent, MtMintEvent};
use near_sdk::{json_types::U128, AccountId, AccountIdRef, CryptoHash};

use crate::contract::Contract;

impl StateView for Contract {
    #[inline]
    fn verifying_contract(&self) -> Cow<'_, AccountIdRef> {
        Cow::Borrowed(CURRENT_ACCOUNT_ID.as_ref())
    }

    #[inline]
    fn wnear_id(&self) -> Cow<'_, AccountIdRef> {
        Cow::Borrowed(self.state.wnear_id.as_ref())
    }

    #[inline]
    fn fee(&self) -> Pips {
        self.state.fees.fee
    }

    #[inline]
    fn fee_collector(&self) -> Cow<'_, AccountIdRef> {
        Cow::Borrowed(self.state.fees.fee_collector.as_ref())
    }

    #[inline]
    fn has_public_key(&self, account_id: &AccountIdRef, public_key: &PublicKey) -> bool {
        self.accounts
            .get(account_id)
            .map(|account| account.has_public_key(account_id, public_key))
            .unwrap_or_else(|| account_id == &public_key.to_implicit_account_id())
    }

    fn iter_public_keys(&self, account_id: &AccountIdRef) -> impl Iterator<Item = PublicKey> + '_ {
        let account = self.accounts.get(account_id);
        account
            .map(|account| account.iter_public_keys(account_id))
            .into_iter()
            .flatten()
            .chain(if account.is_none() {
                PublicKey::from_implicit_account_id(account_id)
            } else {
                None
            })
    }

    #[inline]
    fn is_nonce_used(&self, account_id: &AccountIdRef, nonce: Nonce) -> bool {
        self.accounts
            .get(account_id)
            .map(|account| account.is_nonce_used(nonce))
            .unwrap_or_default()
    }

    #[inline]
    fn balance_of(&self, account_id: &AccountIdRef, token_id: &TokenId) -> u128 {
        self.accounts
            .get(account_id)
            .map(|account| account.token_balances.balance_of(token_id))
            .unwrap_or_default()
    }
}

impl State for Contract {
    #[must_use]
    #[inline]
    fn add_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        self.accounts
            .get_or_create(account_id.clone())
            .add_public_key(&account_id, public_key)
    }

    #[must_use]
    #[inline]
    fn remove_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
        self.accounts
            .get_or_create(account_id.clone())
            .remove_public_key(&account_id, &public_key)
    }

    #[must_use]
    #[inline]
    fn commit_nonce(&mut self, account_id: AccountId, nonce: Nonce) -> bool {
        self.accounts.get_or_create(account_id).commit_nonce(nonce)
    }

    fn internal_deposit(
        &mut self,
        owner_id: AccountId,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        let owner = self.accounts.get_or_create(owner_id);
        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }
            owner
                .token_balances
                .deposit(token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }
        Ok(())
    }

    fn deposit(
        &mut self,
        owner_id: AccountId,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let owner = self.accounts.get_or_create(owner_id.clone());

        let mut mint_event = MtMintEvent {
            owner_id: Cow::Borrowed(owner_id.as_ref()),
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into),
        };

        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }

            mint_event.token_ids.to_mut().push(token_id.to_string());
            mint_event.amounts.to_mut().push(U128(amount));

            self.state
                .total_supplies
                .deposit(token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
            owner
                .token_balances
                .deposit(token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        MtEvent::MtMint([mint_event].as_slice().into()).emit();

        Ok(())
    }

    fn internal_withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()> {
        let owner = self
            .accounts
            .get_mut(owner_id)
            .ok_or(DefuseError::AccountNotFound)?;
        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }

            owner
                .token_balances
                .withdraw(token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }
        Ok(())
    }

    fn withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()> {
        let owner = self
            .accounts
            .get_mut(owner_id)
            .ok_or(DefuseError::AccountNotFound)?;

        let mut burn_event = MtBurnEvent {
            owner_id: Cow::Borrowed(owner_id),
            authorized_id: None,
            token_ids: Default::default(),
            amounts: Default::default(),
            memo: memo.map(Into::into),
        };

        for (token_id, amount) in token_amounts {
            if amount == 0 {
                return Err(DefuseError::ZeroAmount);
            }

            burn_event.token_ids.to_mut().push(token_id.to_string());
            burn_event.amounts.to_mut().push(U128(amount));

            owner
                .token_balances
                .withdraw(token_id.clone(), amount)
                .ok_or(DefuseError::BalanceOverflow)?;
            self.state
                .total_supplies
                .withdraw(token_id, amount)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        MtEvent::MtBurn([burn_event].as_slice().into()).emit();

        Ok(())
    }

    fn on_mt_transfer(&mut self, sender_id: &AccountIdRef, transfer: MtBatchTransfer) {
        todo!()
    }

    fn on_ft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: FtWithdraw) {
        // detach promise
        self.internal_ft_withdraw(owner_id.to_owned(), withdraw);
    }

    fn on_nft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: NftWithdraw) {
        // detach promise
        self.internal_nft_withdraw(owner_id.to_owned(), withdraw);
    }

    fn on_mt_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: MtWithdraw) {
        // detach promise
        self.internal_mt_withdraw(owner_id.to_owned(), withdraw);
    }

    // #[must_use]
    // #[inline]
    // fn internal_add_delta(
    //     &mut self,
    //     owner_id: AccountId,
    //     token_id: TokenId,
    //     delta: i128,
    // ) -> Option<u128> {
    //     self.transfer_runtime
    //         .add_delta(owner_id.clone(), token_id.clone(), delta)?;
    //     self.accounts
    //         .get_or_create(owner_id)
    //         .token_balances
    //         .add_delta(token_id, delta)
    // }

    // fn mt_transfer(
    //     &mut self,
    //     sender_id: AccountId,
    //     MtBatchTransfer {
    //         receiver_id,
    //         token_ids,
    //         amounts,
    //         memo,
    //         msg,
    //     }: MtBatchTransfer,
    // ) -> Result<()> {
    //     if let Some(msg) = msg {
    //         const GAS_FOR_MT_ON_TRANSFER: Gas = Gas::from_tgas(15);

    //         self.internal_mt_batch_transfer_call(
    //             sender_id,
    //             receiver_id,
    //             token_ids,
    //             amounts,
    //             memo.as_deref(),
    //             msg,
    //             GAS_FOR_MT_ON_TRANSFER,
    //         )
    //         // detach promise
    //         .map(|_promise| ())
    //     } else {
    //         self.internal_mt_batch_transfer(
    //             &sender_id,
    //             receiver_id,
    //             token_ids,
    //             amounts,
    //             memo.as_deref(),
    //         )
    //     }
    // }

    // fn ft_withdraw(&mut self, owner_id: AccountId, withdraw: FtWithdraw) -> Result<()> {
    //     self.internal_ft_withdraw(owner_id, withdraw)
    //         // detach promise
    //         .map(|_promise| ())
    // }

    // fn nft_withdraw(&mut self, owner_id: AccountId, withdraw: NftWithdraw) -> Result<()> {
    //     self.internal_nft_withdraw(owner_id, withdraw)
    //         // detach promise
    //         .map(|_promise| ())
    // }

    // fn mt_withdraw(&mut self, owner_id: AccountId, withdraw: MtWithdraw) -> Result<()> {
    //     self.internal_mt_withdraw(owner_id, withdraw)
    //         // detach promise
    //         .map(|_promise| ())
    // }
}
