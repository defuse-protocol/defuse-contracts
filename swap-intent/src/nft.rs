use defuse_contracts::intents::swap::{Asset, NftItem, GAS_FOR_NFT_TRANSFER};
use near_contract_standards::non_fungible_token::{
    core::{ext_nft_core, NonFungibleTokenReceiver},
    TokenId,
};
use near_sdk::{env, near, serde_json, AccountId, NearToken, Promise, PromiseOrValue};

use crate::{SwapIntentContractImpl, SwapIntentContractImplExt};

#[near]
impl NonFungibleTokenReceiver for SwapIntentContractImpl {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        #[allow(unused_variables)] previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let action = serde_json::from_str(&msg).expect("JSON");
        match self
            .handle_action(
                sender_id,
                Asset::Nft(NftItem {
                    collection: env::predecessor_account_id(),
                    token_id,
                }),
                action,
            )
            .unwrap()
        {
            PromiseOrValue::Value(()) => PromiseOrValue::Value(false),
            PromiseOrValue::Promise(promise) => PromiseOrValue::Promise(promise),
        }
    }
}

impl SwapIntentContractImpl {
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
            .with_static_gas(GAS_FOR_NFT_TRANSFER)
            .nft_transfer(recipient, token_id, None, memo.into())
    }
}
