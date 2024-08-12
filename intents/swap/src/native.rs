use defuse_contracts::{
    intents::swap::{
        AssetWithAccount, NativeReceiver, NearAsset, SwapIntentAction, SwapIntentError,
    },
    utils::UnwrapOrPanic,
};
use near_sdk::{env, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl NativeReceiver for SwapIntentContractImpl {
    /// Receive native NEAR.  
    /// `msg` parameter should contain [`SwapIntentAction`] serialized to
    /// JSON string.
    #[payable]
    fn native_on_transfer(&mut self, msg: String) -> PromiseOrValue<bool> {
        self.internal_native_action(&msg).unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_native_action(
        &mut self,
        msg: impl AsRef<str>,
    ) -> Result<PromiseOrValue<bool>, SwapIntentError> {
        let action = serde_json::from_str(msg.as_ref()).map_err(SwapIntentError::JSON)?;

        let received = AssetWithAccount::Near {
            account: env::predecessor_account_id(),
            asset: NearAsset::Native {
                amount: env::attached_deposit(),
            },
        };
        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(received, create)?;
                PromiseOrValue::Value(true)
            }
            SwapIntentAction::Execute(execute) => self.execute_intent(&received, execute)?.into(),
        })
    }

    #[inline]
    pub(crate) fn transfer_native(amount: NearToken, recipient: AccountId) -> Promise {
        // TODO: extend with optional function name and args
        // for function_call() to allow further communication
        // with other protocols.
        // This can lead to potential security flaws where user
        // can call arbitrary contracts. In order to overcome this issue,
        // we can introduce "vault" concept, where all escrow funds for each
        // user are managed by a separate contract, which can be called only
        // by this intent.
        Promise::new(recipient).transfer(amount)
    }
}
