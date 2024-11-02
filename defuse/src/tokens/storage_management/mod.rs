use defuse_contracts::{
    defuse::{intents::tokens::StorageDeposit, tokens::TokenId, Result},
    utils::UnwrapOrPanicError,
    wnear::{ext_wnear, NEAR_WITHDRAW_GAS},
};
use near_sdk::{
    env,
    json_types::U128,
    require,
    serde_json::{self, json},
    AccountId, Gas, NearToken, Promise,
};

use crate::{accounts::Account, state::State};

const STORAGE_DEPOSIT_GAS: Gas = Gas::from_tgas(10);

impl State {
    pub fn storage_deposit(
        &mut self,
        sender_id: &AccountId,
        sender: &mut Account,
        storage_deposit @ StorageDeposit { amount, .. }: StorageDeposit,
    ) -> Result<Promise> {
        self.internal_withdraw(
            sender_id,
            sender,
            [(
                TokenId::Nep141(self.wnear_id.clone()),
                amount.as_yoctonear(),
            )],
            Some("storage_deposit"),
        )?;
        Ok(self.internal_storage_deposit(storage_deposit))
    }

    pub fn internal_storage_deposit(
        &mut self,
        StorageDeposit {
            contract_id,
            account_id,
            amount,
        }: StorageDeposit,
    ) -> Promise {
        // TODO: what if receiver == self
        // TODO: check storage_deposit balance on self
        require!(
            // TODO: mul self storage
            amount.saturating_add(env::storage_byte_cost()) < env::account_balance(),
            "not enough"
        );
        let near_withdraw = ext_wnear::ext(self.wnear_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(NEAR_WITHDRAW_GAS)
            .near_withdraw(U128(amount.as_yoctonear()));
        if contract_id == self.wnear_id {
            // reuse the same promise
            near_withdraw
        } else {
            // near_withdraw will be dropped
            Promise::new(contract_id)
        }
        .function_call(
            "storage_deposit".to_string(),
            serde_json::to_vec(&json!({
                "account_id": Some(account_id),
                // TODO
                "registration_only": Some(false),
            }))
            .unwrap_or_panic_display(),
            amount,
            STORAGE_DEPOSIT_GAS,
        )
    }
}
