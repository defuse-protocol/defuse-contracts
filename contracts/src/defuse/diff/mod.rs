pub mod tokens;

use near_sdk::{ext_contract, near};

use crate::utils::Deadline;

use super::{payload::SignedPayloads, tokens::TokenId, Result};

use self::tokens::TokenDeltas;

#[ext_contract(ext_signed_differ)]
pub trait SignedDiffer {
    #[handle_result]
    fn apply_signed_diffs(&mut self, diffs: SignedPayloads<AccountDiff>) -> Result<()>;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "unit-testing", derive(Default))]
#[near(serializers = [borsh, json])]
pub struct AccountDiff {
    /// For the ease of indexing
    #[serde(default)]
    pub query_id: u64,

    #[serde(default, skip_serializing_if = "TokenDeltas::is_empty")]
    pub tokens: TokenDeltas,

    pub deadline: Deadline,
}

impl AccountDiff {
    #[inline]
    pub fn with_query_id(mut self, query_id: u64) -> Self {
        self.query_id = query_id;
        self
    }

    #[inline]
    pub fn with_tokens<I>(mut self, deltas: I) -> Result<Self>
    where
        I: IntoIterator<Item = (TokenId, i128)>,
    {
        self.tokens.append(deltas)?;
        Ok(self)
    }

    #[inline]
    pub fn with_deadline(mut self, deadline: Deadline) -> Self {
        self.deadline = deadline;
        self
    }
}
