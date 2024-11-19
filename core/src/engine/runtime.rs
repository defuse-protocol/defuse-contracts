use std::{collections::HashMap, mem};

use crate::{
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};

#[derive(Debug, Default)]
pub struct Runtime {
    pub total_supply_deltas: TokenAmounts<HashMap<TokenId, i128>>,
}

impl Runtime {
    #[inline]
    pub fn finalize(mut self) -> Result<()> {
        if !mem::take(&mut self).total_supply_deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        Ok(())
    }
}
