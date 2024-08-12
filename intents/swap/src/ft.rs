use defuse_contracts::{
    intents::swap::{AssetWithAccount, FtAmount, NearAsset, SwapIntentAction, SwapIntentError},
    utils::UnwrapOrPanic,
};
use near_contract_standards::fungible_token::{core::ext_ft_core, receiver::FungibleTokenReceiver};
use near_sdk::{
    env, json_types::U128, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue,
};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl FungibleTokenReceiver for SwapIntentContractImpl {
    /// Receive NEP-141 tokens.  
    /// `msg` parameter should contain [`SwapIntentAction`] serialized to
    /// JSON string.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.internal_ft_on_transfer(sender_id, amount, msg)
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: impl AsRef<str>,
    ) -> Result<PromiseOrValue<U128>, SwapIntentError> {
        let action = serde_json::from_str(msg.as_ref()).map_err(SwapIntentError::JSON)?;

        let received = AssetWithAccount::Near {
            account: sender_id,
            asset: NearAsset::Nep141(FtAmount {
                token: env::predecessor_account_id(),
                amount,
            }),
        };

        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(received, create)?;
                // intent was successfully created, do not refund
                PromiseOrValue::Value(0.into())
            }
            SwapIntentAction::Execute(execute) => self.execute_intent(&received, execute)?.into(),
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
            .with_static_gas(NearAsset::GAS_FOR_FT_TRANSFER)
            .ft_transfer(recipient, amount, memo.into())
    }
}
