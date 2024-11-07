use defuse_contracts::{
    intents::swap::{
        AssetWithAccount, CrossChainAsset, CrossChainReceiver, SwapIntentAction, SwapIntentError,
    },
    utils::UnwrapOrPanicError,
};
use near_sdk::{env, near, serde_json, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl CrossChainReceiver for SwapIntentContractImpl {
    /// Receive cross-chain asset.  
    /// `msg` parameter should contain [`SwapIntentAction`] serialized to
    /// JSON string.
    fn cross_chain_on_transfer(
        &mut self,
        asset: String,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.internal_cross_chain_action(asset, amount, msg)
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    #[allow(clippy::needless_pass_by_value)]
    fn internal_cross_chain_action(
        &mut self,
        cross_chain_asset: String,
        amount: String,
        msg: String,
    ) -> Result<PromiseOrValue<bool>, SwapIntentError> {
        let action = serde_json::from_str(&msg).map_err(SwapIntentError::JSON)?;

        let received = AssetWithAccount::CrossChain {
            account: String::new(),
            asset: CrossChainAsset {
                oracle: env::predecessor_account_id(),
                asset: cross_chain_asset,
                amount,
            },
        };
        match action {
            SwapIntentAction::Create(create) => self
                .create_intent(received, create)
                .map(|()| PromiseOrValue::Value(true)),
            SwapIntentAction::Execute(execute) => {
                self.execute_intent(&received, execute).map(Into::into)
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn transfer_cross_chain_asset(
        _asset: CrossChainAsset,
        _recipient: String,
    ) -> Promise {
        // TODO: send a request to the oracle for the other chain to
        // transfer escrow assets to the recipient.
        // For now, this is a placeholder
        Promise::new(env::current_account_id())
    }
}
