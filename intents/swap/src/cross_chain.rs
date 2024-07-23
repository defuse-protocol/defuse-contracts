use defuse_contracts::intents::swap::{
    AssetWithAccount, CrossChainAsset, CrossChainTransferReceiver, SwapIntentAction,
    SwapIntentError,
};
use near_sdk::{env, near, serde_json, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl CrossChainTransferReceiver for SwapIntentContractImpl {
    fn on_cross_chain_transfer(
        &mut self,
        asset: String,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.internal_cross_chain_action(asset, amount, msg)
            .unwrap()
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
        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(received, create)?;
                PromiseOrValue::Value(true)
            }
            SwapIntentAction::Execute(execute) => self.execute_intent(&received, execute)?.into(),
        })
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
