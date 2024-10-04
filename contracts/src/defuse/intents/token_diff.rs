use impl_tools::autoimpl;
use near_sdk::near;

use crate::defuse::tokens::TokenAmounts;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default)]
#[autoimpl(Deref using self.diff)]
#[autoimpl(DerefMut using self.diff)]
pub struct TokenDiff {
    #[serde(default, skip_serializing_if = "TokenAmounts::is_empty")]
    pub diff: TokenAmounts<i128>,
}
