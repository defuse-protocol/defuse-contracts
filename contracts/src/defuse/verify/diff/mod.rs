pub mod tokens;

use near_sdk::{near, AccountId};

use std::collections::HashMap;

use tokens::TokenDeltas;

use crate::{
    defuse::{error::DefuseError, token::TokenId},
    nep413::SignedPayload,
    utils::Deadline,
};

/// Each signer can have multiple diffs signed
pub type SignedDiffs = HashMap<AccountId, Vec<SignedPayload<AccountDiff>>>;

#[derive(Debug, Clone, Default)]
#[near(serializers = [borsh, json])]
pub struct AccountDiff {
    /// For the ease of indexing
    #[serde(default)]
    pub query_id: u64,

    #[serde(default, skip_serializing_if = "TokenDeltas::is_empty")]
    pub tokens: TokenDeltas,

    #[serde(default, skip_serializing_if = "Deadline::is_none")]
    pub deadline: Deadline,
}

impl AccountDiff {
    #[inline]
    pub fn with_query_id(mut self, query_id: u64) -> Self {
        self.query_id = query_id;
        self
    }

    #[inline]
    pub fn with_tokens<I>(mut self, deltas: I) -> Result<Self, DefuseError>
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
