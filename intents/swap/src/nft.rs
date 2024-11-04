use defuse_contracts::{
    intents::swap::{AssetWithAccount, NearAsset, NftItem, SwapIntentAction, SwapIntentError},
    utils::UnwrapOrPanicError,
};
use near_contract_standards::non_fungible_token::{
    core::{ext_nft_core, NonFungibleTokenReceiver},
    TokenId,
};
use near_sdk::{env, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl NonFungibleTokenReceiver for SwapIntentContractImpl {
    /// Receive NEP-141 tokens.  
    /// `msg` parameter should contain [`SwapIntentAction`] serialized to
    /// JSON string.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        self.internal_nft_on_transfer(sender_id, &previous_owner_id, token_id, msg)
            .unwrap_or_panic_display()
    }
}

impl SwapIntentContractImpl {
    fn internal_nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        #[allow(unused_variables)] previous_owner_id: &AccountId,
        token_id: TokenId,
        msg: impl AsRef<str>,
    ) -> Result<PromiseOrValue<bool>, SwapIntentError> {
        let action = serde_json::from_str(msg.as_ref()).map_err(SwapIntentError::JSON)?;

        let received = AssetWithAccount::Near {
            account: sender_id,
            asset: NearAsset::Nep171(NftItem {
                collection: env::predecessor_account_id(),
                token_id,
            }),
        };

        Ok(match action {
            SwapIntentAction::Create(create) => {
                self.create_intent(received, create)?;
                // intent was successfully created, do not refund
                PromiseOrValue::Value(false)
            }
            SwapIntentAction::Execute(execute) => self.execute_intent(&received, execute)?.into(),
        })
    }

    #[inline]
    pub(crate) fn transfer_nft(
        NftItem {
            collection,
            token_id,
        }: NftItem,
        recipient: AccountId,
        memo: impl Into<Option<String>>,
    ) -> Promise {
        // TODO: extend with optional msg for nft_transfer_call()
        // for extensibility to allow further communication with other
        // protocols
        ext_nft_core::ext(collection)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(NearAsset::GAS_FOR_NFT_TRANSFER)
            .nft_transfer(recipient, token_id, None, memo.into())
    }
}
