use impl_tools::autoimpl;
use near_sdk::near;

use crate::{
    defuse::tokens::TokenAmounts,
    utils::{fees::Pips, integer::CheckedMulDiv},
};

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default)]
#[autoimpl(Deref using self.diff)]
#[autoimpl(DerefMut using self.diff)]
pub struct TokenDiff {
    #[serde(default, skip_serializing_if = "TokenAmounts::is_empty")]
    pub diff: TokenAmounts<i128>,
}

impl TokenDiff {
    #[inline]
    pub fn estimate_amount_out(amount_in: u128, fee: Pips) -> u128 {
        // amount_out = ⌊⌊amount_in * (1-fee)⌋ / (1+fee)⌋
        fee.invert()
            .fee(amount_in)
            .checked_mul_div(
                Pips::MAX.as_pips() as u128,
                Pips::MAX.as_pips() as u128 + fee.as_pips() as u128,
            )
            .unwrap_or_else(|| unreachable!())
    }

    #[inline]
    pub fn estimate_amount_in(amount_out: u128, fee: Pips) -> Option<u128> {
        // amount_in = ⌈⌈amount_out * (1+fee)⌉ / (1-fee)⌉
        amount_out
            .checked_add(fee.fee_ceil(amount_out))?
            .checked_mul_div_ceil(Pips::MAX.as_pips() as u128, fee.invert().as_pips() as u128)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[test]
    fn estimate_amount_in(
        #[values(1_000_000)] amount: u128,
        #[values(
            Pips::ZERO,
            Pips::ONE_PIP,
            Pips::ONE_BIP,
            Pips::ONE_PERCENT,
            Pips::ONE_PERCENT * 50,
        )]
        fee: Pips,
    ) {
        let amount_in = TokenDiff::estimate_amount_in(amount, fee).unwrap();
        check_invariant(amount_in as i128, amount as i128, fee);
    }

    #[rstest]
    #[test]
    fn estimate_amount_out(
        #[values(1_000_000)] amount: u128,
        #[values(
            Pips::ZERO,
            Pips::ONE_PIP,
            Pips::ONE_BIP,
            Pips::ONE_PERCENT,
            Pips::ONE_PERCENT * 50,
        )]
        fee: Pips,
    ) {
        let amount_out = TokenDiff::estimate_amount_out(amount, fee);
        check_invariant(amount as i128, amount_out as i128, fee);
    }

    #[track_caller]
    fn check_invariant(amount_in: i128, amount_out: i128, fee: Pips) {
        assert_eq!(
            -amount_in
                + fee.fee_ceil(amount_in.unsigned_abs()) as i128
                + amount_out
                + fee.fee_ceil(amount_out.unsigned_abs()) as i128,
            0,
            "invariant violated: amount_in: {amount_in}, amount_out: {amount_out}, fee: {fee}",
        );
    }
}
