use defuse_contracts::{
    intents::swap::{AssetWithAccount, MultiToken, NearAsset, SwapIntentAction, SwapIntentError},
    mt::{
        core::{ext_mt_core, MultiTokenReceiver},
        TokenId,
    },
};
use near_sdk::{
    env, json_types::U128, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue,
};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl MultiTokenReceiver for SwapIntentContractImpl {
    fn mt_on_transfer(
        &mut self,
        sender_id: AccountId,
        #[allow(unused_variables)] previous_owner_ids: Vec<AccountId>,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.internal_mt_on_transfer(sender_id, token_ids, amounts, msg)
            .unwrap()
    }
}

impl SwapIntentContractImpl {
    fn internal_mt_on_transfer(
        &mut self,
        sender_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        msg: impl AsRef<str>,
    ) -> Result<PromiseOrValue<Vec<U128>>, SwapIntentError> {
        let action = serde_json::from_str(msg.as_ref()).map_err(SwapIntentError::JSON)?;

        let num_token_ids = token_ids.len();
        let received = AssetWithAccount::Near {
            account: sender_id,
            asset: NearAsset::Nep245(MultiToken {
                contract: env::predecessor_account_id(),
                token_ids,
                amounts,
            }),
        };

        match action {
            SwapIntentAction::Create(create) => self
                .create_intent(received, create)
                .map(|_| PromiseOrValue::Value(vec![U128(0); num_token_ids])),
            SwapIntentAction::Execute(execute) => {
                self.execute_intent(&received, execute).map(Into::into)
            }
        }
    }

    pub(crate) fn transfer_mt(
        MultiToken {
            contract,
            token_ids,
            amounts,
        }: MultiToken,
        recipient: AccountId,
        memo: impl Into<Option<String>>,
    ) -> Promise {
        // TODO: extend with optional msg for mt_batch_transfer_call()
        // for extensibility to allow further communication with other
        // protocols
        ext_mt_core::ext(contract)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(NearAsset::GAS_FOR_MT_TRANSFER)
            .mt_batch_transfer(recipient, token_ids, amounts, None, memo.into())
    }
}
