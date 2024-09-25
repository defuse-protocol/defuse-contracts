use defuse_contracts::nep245::{resolver::MultiTokenResolver, TokenId};
use near_sdk::{json_types::U128, near, AccountId};

use crate::{DefuseImpl, DefuseImplExt};

#[near]
impl MultiTokenResolver for DefuseImpl {
    #[private]
    fn mt_resolve_transfer(
        &mut self,
        previous_owner_ids: Vec<AccountId>,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        // approvals: Option<Vec<Option<Vec<ClearedApproval>>>>,
    ) -> Vec<U128> {
        todo!()
    }
}
