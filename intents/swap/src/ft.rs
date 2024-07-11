use defuse_contracts::intents::swap::{Asset, FtAmount, SwapIntentAction, SwapIntentError};
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
        self.internal_ft_on_transfer(sender_id, amount, msg)
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: impl AsRef<str>,
    ) -> Result<PromiseOrValue<U128>, SwapIntentError> {
        if amount.0 == 0 {
            return Err(SwapIntentError::ZeroAmount);
        }
        let action = serde_json::from_str(msg.as_ref()).map_err(SwapIntentError::JSON)?;

        let received = Asset::Ft(FtAmount {
            token: env::predecessor_account_id(),
            amount,
        });

        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(sender_id, received, create)?;
                // intent was successfully created, do not refund
                PromiseOrValue::Value(0.into())
            }
            SwapIntentAction::Execute(execute) => {
                self.execute_intent(sender_id, received, execute)?.into()
            }
        })
    }

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
            .with_static_gas(Asset::GAS_FOR_FT_TRANSFER)
            .ft_transfer(recipient, amount, memo.into())
    }
}
