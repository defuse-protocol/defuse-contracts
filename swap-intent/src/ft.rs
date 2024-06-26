use defuse_contracts::intents::swap::{Asset, FtAmount, GAS_FOR_FT_TRANSFER};
use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{
    env, json_types::U128, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue,
};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl FungibleTokenReceiver for SwapIntentContractImpl {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let action = serde_json::from_str(&msg).expect("JSON");

        match self
            .handle_action(
                sender_id,
                Asset::Ft(FtAmount {
                    token: env::predecessor_account_id(),
                    amount: amount.0,
                }),
                action,
            )
            .unwrap()
        {
            // TODO: separate callback
            PromiseOrValue::Value(()) => PromiseOrValue::Value(0.into()),
            PromiseOrValue::Promise(promise) => PromiseOrValue::Promise(promise),
        }
    }
}

impl SwapIntentContractImpl {
    #[inline]
    pub(crate) fn transfer_ft(
        FtAmount { token, amount }: FtAmount,
        recipient: AccountId,
        memo: impl Into<Option<String>>,
    ) -> Promise {
        // TODO: extend with optional msg for ft_transfer_call()
        // for extensibility to allow further communication with other
        // protocols
        ext_ft_core::ext(token)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(recipient, amount.into(), memo.into())
    }
}
