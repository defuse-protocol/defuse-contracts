mod account;
mod nonces;
mod state;

pub use self::{account::*, nonces::*, state::*};

use std::collections::HashSet;

use defuse_contracts::{
    crypto::PublicKey,
    defuse::{
        accounts::AccountManager, message::SignedDefuseMessage, payload::ValidatePayloadAs,
        DefuseError, Result,
    },
    nep413::{Nep413Payload, Nonce},
    utils::{
        cache::{CURRENT_ACCOUNT_ID, PREDECESSOR_ACCOUNT_ID},
        prefix::NestPrefix,
        serde::wrappers::DisplayFromStr,
    },
};

use near_plugins::{pause, Pausable};
use near_sdk::{
    borsh::BorshSerialize, near, store::IterableMap, AccountId, BorshStorageKey, IntoStorageKey,
};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl AccountManager for DefuseImpl {
    fn has_public_key(&self, account_id: &AccountId, public_key: &PublicKey) -> bool {
        self.accounts
            .get(account_id)
            .map(|account| account.has_public_key(account_id, public_key))
            .unwrap_or_else(|| account_id == &public_key.to_implicit_account_id())
    }

    fn public_keys_of(&self, account_id: &AccountId) -> HashSet<PublicKey> {
        self.accounts
            .get(account_id)
            .into_iter()
            .flat_map(Account::iter_public_keys)
            .cloned()
            .collect()
    }

    #[pause(name = "accounts")]
    fn add_public_key(&mut self, public_key: PublicKey) {
        self.accounts
            .get_or_create(PREDECESSOR_ACCOUNT_ID.clone())
            .add_public_key(&PREDECESSOR_ACCOUNT_ID, public_key);
    }

    #[pause(name = "accounts")]
    fn remove_public_key(&mut self, public_key: &PublicKey) {
        self.accounts
            // create account if doesn't exist, so the user can opt out of implicit public key
            .get_or_create(PREDECESSOR_ACCOUNT_ID.clone())
            .remove_public_key(&PREDECESSOR_ACCOUNT_ID, public_key);
    }

    fn is_nonce_used(&self, account_id: &AccountId, nonce: DisplayFromStr<Nonce>) -> bool {
        self.accounts
            .get(account_id)
            .map(move |account| account.is_nonce_used(nonce.into_inner()))
            .unwrap_or_default()
    }

    fn find_unused_nonce(
        &self,
        account_id: &AccountId,
        start: Option<DisplayFromStr<Nonce>>,
    ) -> Option<DisplayFromStr<Nonce>> {
        let start = start.map(DisplayFromStr::into_inner);
        if let Some(account) = self.accounts.get(account_id) {
            account.find_unused_nonce(start)
        } else {
            Some(start.unwrap_or_default())
        }
        .map(Into::into)
    }

    #[pause(name = "accounts")]
    #[handle_result]
    fn invalidate_nonces(&mut self, nonces: Vec<DisplayFromStr<Nonce>>) {
        let account = self.accounts.get_or_create(PREDECESSOR_ACCOUNT_ID.clone());
        for n in nonces.into_iter().map(DisplayFromStr::into_inner) {
            let _ = account.commit_nonce(n);
        }
    }
}

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct Accounts {
    accounts: IterableMap<AccountId, Account>,
    prefix: Vec<u8>,
}

impl Accounts {
    #[inline]
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let prefix = prefix.into_storage_key();
        Self {
            accounts: IterableMap::new(prefix.as_slice().nest(AccountsPrefix::Accounts)),
            prefix,
        }
    }

    #[inline]
    pub fn get(&self, account_id: &AccountId) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    #[inline]
    pub fn get_mut(&mut self, account_id: &AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(account_id)
    }

    #[inline]
    pub fn get_or_create(&mut self, account_id: AccountId) -> &mut Account {
        self.accounts
            .entry(account_id)
            .or_insert_with_key(|account_id| {
                Account::new(
                    self.prefix
                        .as_slice()
                        .nest(AccountsPrefix::Account(account_id)),
                    account_id,
                )
            })
    }

    pub fn verify_signed_message<T>(
        &mut self,
        signed: SignedDefuseMessage<T>,
    ) -> Result<(AccountId, &mut Account, T)>
    where
        T: BorshSerialize,
    {
        // verify signature and derive its public key
        let public_key = signed.verify().ok_or(DefuseError::InvalidSignature)?;

        // extract NEP-413 payload
        let payload: Nep413Payload<_> = signed.payload.validate_as()?;

        // check recipient
        if payload.recipient != *CURRENT_ACCOUNT_ID {
            return Err(DefuseError::WrongRecipient);
        }
        // check deadline
        if payload.deadline.has_expired() {
            return Err(DefuseError::DeadlineExpired);
        }

        let signer_id = payload.message.signer_id;
        let account = self.get_or_create(signer_id.clone());
        // make sure the account has this public key
        if !account.has_public_key(&signer_id, &public_key) {
            return Err(DefuseError::InvalidSignature);
        }
        // commit nonce
        account.commit_nonce(payload.nonce)?;

        Ok((signer_id, account, payload.message.message))
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "::near_sdk::borsh")]
enum AccountsPrefix<'a> {
    Accounts,
    Account(&'a AccountId),
}
