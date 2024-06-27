use defuse_contracts::intents::swap::{Asset, NativeAction, SwapError, SwapIntentAction};
use near_sdk::{env, near, AccountId, NearToken, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl NativeAction for SwapIntentContractImpl {
    #[payable]
    fn native_action(&mut self, action: SwapIntentAction) -> PromiseOrValue<bool> {
        self.internal_native_action(action).unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_native_action(
        &mut self,
        action: SwapIntentAction,
    ) -> Result<PromiseOrValue<bool>, SwapError> {
        let amount = env::attached_deposit();
        if amount.is_zero() {
            return Err(SwapError::ZeroAmount);
        }
        let sender = env::predecessor_account_id();
        let received = Asset::Native(amount);
        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(sender, received, create)?;
                PromiseOrValue::Value(true)
            }
            SwapIntentAction::Execute(execute) => {
                self.execute_intent(sender, received, execute)?.into()
            }
        })
    }

    #[inline]
    pub(crate) fn transfer_native(amount: NearToken, recipient: AccountId) -> Promise {
        // TODO: extend with optional function name and args
        // for function_call() to allow further communication
        // with other protocols
        Promise::new(recipient).transfer(amount)
    }
}
