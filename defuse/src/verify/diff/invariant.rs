use defuse_contracts::defuse::{token::TokenId, verify::diff::tokens::TokenDeltas, DefuseError};

#[derive(Debug, Default)]
pub struct Invariant {
    token_deltas: TokenDeltas,
}

impl Invariant {
    #[inline]
    pub fn on_token_delta(&mut self, token_id: TokenId, delta: i128) -> Result<(), DefuseError> {
        self.token_deltas.add_delta(token_id, delta)?;
        Ok(())
    }

    #[inline]
    pub fn ensure(self) -> Result<(), DefuseError> {
        if !self.token_deltas.is_empty() {
            return Err(DefuseError::InvariantViolated);
        }
        Ok(())
    }
}
