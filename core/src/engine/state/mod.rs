pub mod cached;

use std::borrow::Cow;

use cached::CachedState;
use defuse_crypto::PublicKey;
use impl_tools::autoimpl;
use near_sdk::{AccountId, AccountIdRef};

use crate::{
    fees::Pips,
    intents::tokens::{FtWithdraw, MtBatchTransfer, MtWithdraw, NftWithdraw},
    tokens::TokenId,
    Nonce, Result,
};

#[autoimpl(for<T: trait + ?Sized> &T, &mut T, Box<T>)]
pub trait StateView {
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

    #[must_use]
    fn internal_add_delta(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        delta: i128,
    ) -> Option<u128>;

    fn mt_transfer(&mut self, sender_id: AccountId, transfer: MtBatchTransfer) -> Result<()>;

    fn ft_withdraw(&mut self, owner_id: AccountId, withdraw: FtWithdraw) -> Result<()>;

    fn nft_withdraw(&mut self, owner_id: AccountId, withdraw: NftWithdraw) -> Result<()>;

    fn mt_withdraw(&mut self, owner_id: AccountId, withdraw: MtWithdraw) -> Result<()>;
}
