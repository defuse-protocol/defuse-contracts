pub mod cached;

use std::borrow::Cow;

use cached::CachedState;
use defuse_crypto::PublicKey;
use impl_tools::autoimpl;
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    fees::Pips,
    intents::tokens::{FtWithdraw, MtWithdraw, NativeWithdraw, NftWithdraw},
    tokens::TokenId,
    Nonce, Result,
};

#[autoimpl(for<T: trait + ?Sized> &T, &mut T, Box<T>)]
pub trait StateView {
    // TODO: remove?
    fn verifying_contract(&self) -> Cow<'_, AccountIdRef>;
    fn wnear_id(&self) -> Cow<'_, AccountIdRef>;

    fn fee(&self) -> Pips;
    fn fee_collector(&self) -> Cow<'_, AccountIdRef>;

    fn has_public_key(&self, account_id: &AccountIdRef, public_key: &PublicKey) -> bool;
    fn iter_public_keys(&self, account_id: &AccountIdRef) -> impl Iterator<Item = PublicKey> + '_;

    fn is_nonce_used(&self, account_id: &AccountIdRef, nonce: Nonce) -> bool;

    fn balance_of(&self, account_id: &AccountIdRef, token_id: &TokenId) -> u128;

    fn cached(self) -> CachedState<Self>
    where
        Self: Sized,
    {
        CachedState::new(self)
    }
}

#[autoimpl(for<T: trait + ?Sized> &mut T, Box<T>)]
pub trait State: StateView {
    // TODO: Cow
    #[must_use]
    fn add_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool;
    #[must_use]
    fn remove_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool;

    #[must_use]
    fn commit_nonce(&mut self, account_id: AccountId, nonce: Nonce) -> bool;

    fn internal_deposit(
        &mut self,
        owner_id: AccountId,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()>;
    // TODO: docs
    fn deposit(
        &mut self,
        owner_id: AccountId,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()>;
    fn internal_withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
    ) -> Result<()>;
    // TODO: docs
    fn withdraw(
        &mut self,
        owner_id: &AccountIdRef,
        tokens: impl IntoIterator<Item = (TokenId, u128)>,
        memo: Option<&str>,
    ) -> Result<()>;

    fn on_ft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: FtWithdraw);
    fn on_nft_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: NftWithdraw);
    fn on_mt_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: MtWithdraw);
    fn on_native_withdraw(&mut self, owner_id: &AccountIdRef, withdraw: NativeWithdraw);
}
