use impl_tools::autoimpl;
use near_sdk::{AccountIdRef, CryptoHash};

use crate::{intents::token_diff::TokenDiff, Deadline};

#[autoimpl(for <T: trait + ?Sized> &mut T, Box<T>)]
pub trait Inspector {
    fn on_deadline(&mut self, deadline: Deadline);

    // fn on_mint(
    //     &mut self,
    //     owner_id: &AccountIdRef,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    //     memo: Option<&str>,
    // );
    // fn on_burn(
    //     &mut self,
    //     owner_id: &AccountIdRef,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    //     memo: Option<&str>,
    // );
    // fn on_transfer(
    //     &mut self,
    //     sender_id: &AccountIdRef,
    //     receiver_id: &AccountIdRef,
    //     token_amounts: impl IntoIterator<Item = (TokenId, u128)>,
    //     memo: Option<&str>,
    // );

    fn on_token_diff(&mut self, owner_id: &AccountIdRef, token_diff: &TokenDiff);

    fn on_intent_executed(&mut self, signer_id: &AccountIdRef, hash: CryptoHash);
}
