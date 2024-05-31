use near_sdk::store::LookupMap;
use near_sdk::{near, AccountId, IntoStorageKey};
use std::collections::HashMap;

use crate::error::Error;
use crate::types::Account;

// Accounts that belong user. Key here is derivation path.
type UserAccounts = HashMap<String, Account>;

#[near(serializers=[borsh])]
pub struct AccountDb(LookupMap<AccountId, UserAccounts>);

impl AccountDb {
    pub fn new<S: IntoStorageKey>(prefix: S) -> Self {
        Self(LookupMap::new(prefix))
    }

    pub fn add_account(
        &mut self,
        account_id: AccountId,
        derivation_path: String,
        account: Account,
    ) -> Result<(), Error> {
        if let Some(accounts) = self.0.get_mut(&account_id) {
            if accounts.contains_key(&derivation_path) {
                return Err(Error::AccountExist);
            }

            accounts.insert(derivation_path, account);
        } else {
            self.0
                .insert(account_id, HashMap::from([(derivation_path, account)]));
        }

        Ok(())
    }

    pub fn change_owner(
        &mut self,
        from: &AccountId,
        to: AccountId,
        derivation_path: String,
    ) -> Result<(), Error> {
        let account = self
            .0
            .get_mut(from)
            .ok_or(Error::EmptyAccounts)
            .and_then(|accounts| accounts.remove(&derivation_path).ok_or(Error::NoAccount))?;

        self.add_account(to, derivation_path, account)
    }

    pub fn get_accounts(&self, account_id: &AccountId) -> Result<Vec<(String, Account)>, Error> {
        self.0
            .get(account_id)
            .map_or(Err(Error::EmptyAccounts), |accounts| {
                Ok(accounts
                    .iter()
                    .map(|(d, a)| (d.clone(), a.clone()))
                    .collect())
            })
    }

    #[allow(dead_code)]
    pub fn assert_owner(&self, account_id: &AccountId, derivation_path: &str) -> Result<(), Error> {
        self.0
            .get(account_id)
            .and_then(|accounts| {
                if accounts.contains_key(derivation_path) {
                    Some(())
                } else {
                    None
                }
            })
            .ok_or(Error::EmptyAccounts)
    }
}

#[test]
fn test_account_db_add_account() {
    let mut db = AccountDb::new(1);
    let account_id: AccountId = "test.near".parse().unwrap();
    let result = db.add_account(account_id.clone(), "path".to_string(), Account::default());

    assert!(result.is_ok());
    assert_eq!(
        db.get_accounts(&account_id).unwrap(),
        vec![("path".to_string(), Account::default())]
    );
}

#[test]
fn test_account_db_change_owner() {
    let mut db = AccountDb::new(1);
    let account_id: AccountId = "test.near".parse().unwrap();
    let path = "path".to_string();
    let result = db.add_account(account_id.clone(), path.clone(), Account::default());

    assert!(result.is_ok());
    assert!(matches!(db.assert_owner(&account_id, &path), Ok(())));

    let new_owner: AccountId = "owner.near".parse().unwrap();

    assert!(db
        .change_owner(&account_id, new_owner.clone(), path.clone())
        .is_ok());

    assert!(matches!(
        db.assert_owner(&account_id, &path),
        Err(Error::EmptyAccounts)
    ));
    assert!(matches!(db.assert_owner(&new_owner, &path), Ok(())));
}
