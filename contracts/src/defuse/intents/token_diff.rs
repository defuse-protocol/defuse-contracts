use impl_tools::autoimpl;
use near_sdk::{near, serde::Serialize, AccountId};

use crate::{
    defuse::{
        tokens::{TokenAmounts, TokenId},
        DefuseError, Result,
    },
    utils::{fees::Pips, integer::CheckedMulDiv},
};

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[autoimpl(Deref using self.diff)]
#[autoimpl(DerefMut using self.diff)]
pub struct TokenDiff {
    pub diff: TokenAmounts<i128>,
}

impl TokenDiff {
    /// Returns [`TokenDiff`] closure to successfully execute `self`
    /// assuming given `fee`
    #[inline]
    pub fn closure(self, fee: Pips) -> Result<Self> {
        Self::closure_deltas(self.diff, fee).map(|diff| Self { diff })
    }

    /// Returns [`TokenDiff`] closure to successfully execute given set
    /// of distinct [`TokenDiff`] assuming given `fee`
    #[inline]
    pub fn closure_many(diffs: impl IntoIterator<Item = Self>, fee: Pips) -> Result<Self> {
        Self::closure_deltas(diffs.into_iter().flat_map(|d| d.diff), fee).map(|diff| Self { diff })
    }

    /// Returns closure for deltas that should be given in a single
    /// [`TokenDiff`] to successfully execute given set of distinct `deltas`
    /// assuming given `fee`
    pub fn closure_deltas(
        deltas: impl IntoIterator<Item = (TokenId, i128)>,
        fee: Pips,
    ) -> Result<TokenAmounts<i128>> {
        deltas
            .into_iter()
            // accumulate deltas with fees
            .try_fold(
                TokenAmounts::<i128>::default(),
                |mut deltas, (token_id, delta)| {
                    deltas
                        .add(
                            token_id,
                            Self::delta_in_with_fee(delta, fee)
                                .ok_or(DefuseError::IntegerOverflow)?,
                        )
                        .map(|_| deltas)
                },
            )?
            .into_iter()
            // calculate closure
            .try_fold(TokenAmounts::default(), |mut deltas, (token_id, delta)| {
                deltas
                    .add(
                        token_id,
                        Self::delta_out(delta, fee).ok_or(DefuseError::IntegerOverflow)?,
                    )
                    .map(|_| deltas)
            })
    }

    /// Returns closure for delta that should be given in a single
    /// [`TokenDiff`] to successfully execute [`TokenDiff`] with given
    /// `delta` on the same token assuming given `fee`.
    ///
    /// Formula: `⌊-(delta + ⌈|delta| * fee⌉) / (1 - fee * sign(delta))⌋`
    #[inline]
    pub fn closure_delta(delta: i128, fee: Pips) -> Option<i128> {
        Self::delta_out(Self::delta_in_with_fee(delta, fee)?, fee)
    }

    /// Returns `delta + ⌈|delta| * fee⌉`
    #[inline]
    pub fn delta_in_with_fee(delta: i128, fee: Pips) -> Option<i128> {
        delta.checked_add_unsigned(fee.fee_ceil(delta.unsigned_abs()))
    }

    /// Returns `⌊-delta / (1 - fee * sign(delta))⌋`
    #[inline]
    pub fn delta_out(delta: i128, fee: Pips) -> Option<i128> {
        delta.checked_neg()?.checked_mul_div_euclid(
            Pips::MAX.as_pips() as i128,
            Pips::MAX.as_pips() as i128 - delta.signum() * fee.as_pips() as i128,
        )
    }
}

#[must_use = "make sure to `.emit()` this event"]
#[derive(Debug, Serialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct TokenDiffEvent<'a> {
    pub signer_id: &'a AccountId,

    #[serde(flatten)]
    pub token_diff: &'a TokenDiff,
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

    use crate::defuse::tokens::TokenId;

    use super::*;

    #[rstest]
    #[test]
    fn closure_delta(
        #[values(1_000_000, -1_000_000)] delta: i128,
        #[values(
            Pips::ZERO,
            Pips::ONE_PIP,
            Pips::ONE_BIP,
            Pips::ONE_PERCENT,
            Pips::ONE_PERCENT * 50,
        )]
        fee: Pips,
    ) {
        let closure = TokenDiff::closure_delta(delta, fee).unwrap();

        assert_eq!(
            delta
                + fee.fee_ceil(delta.unsigned_abs()) as i128
                + closure
                + fee.fee_ceil(closure.unsigned_abs()) as i128,
            0,
            "invariant violated: delta: {delta}, closure: {closure}, fee: {fee}",
        );
    }

    #[test]
    fn closure_deltas_empty() {
        assert!(TokenDiff::closure_deltas(None, Pips::ONE_BIP)
            .unwrap()
            .is_empty())
    }

    #[rstest]
    #[test]
    fn closure_deltas_nonoverlapping(
        #[values(
            Pips::ZERO,
            Pips::ONE_PIP,
            Pips::ONE_BIP,
            Pips::ONE_PERCENT,
            Pips::ONE_PERCENT * 50,
        )]
        fee: Pips,
    ) {
        let [t1, t2, t3] = ["ft1", "ft2", "ft3"].map(|t| TokenId::Nep141(t.parse().unwrap()));

        for (d1, d2, d3) in [0, 1, -1, 50, -50, 100, -100, 300, -300]
            .into_iter()
            .tuple_combinations()
        {
            assert_eq!(
                TokenDiff::closure_deltas(
                    [
                        TokenAmounts::default()
                            .with_try_extend::<i128>([(t1.clone(), d1), (t2.clone(), d2)])
                            .unwrap(),
                        TokenAmounts::default()
                            .with_try_extend::<i128>([(t3.clone(), d3)])
                            .unwrap(),
                    ]
                    .into_iter()
                    .flatten(),
                    fee
                )
                .unwrap(),
                TokenAmounts::default()
                    .with_try_extend::<i128>([
                        (t1.clone(), TokenDiff::closure_delta(d1, fee).unwrap()),
                        (t2.clone(), TokenDiff::closure_delta(d2, fee).unwrap()),
                        (t3.clone(), TokenDiff::closure_delta(d3, fee).unwrap()),
                    ])
                    .unwrap(),
                "d1: {d1}, d2: {d2}, d3: {d3}"
            )
        }
    }

    #[rstest]
    #[test]
    fn arbitrage_means_somebody_looses(#[values(Pips::ZERO, Pips::ONE_BIP)] fee: Pips) {
        let [t1, t2, t3] = ["ft1", "ft2", "ft3"].map(|t| TokenId::Nep141(t.parse().unwrap()));

        let closure = TokenDiff::closure_deltas(
            [
                TokenAmounts::default()
                    .with_try_extend::<i128>([(t1.clone(), -100), (t2.clone(), 200)])
                    .unwrap(),
                TokenAmounts::default()
                    .with_try_extend::<i128>([(t2.clone(), -200), (t3.clone(), 300)])
                    .unwrap(),
                TokenAmounts::default()
                    .with_try_extend::<i128>([(t3.clone(), -300), (t1.clone(), 101)])
                    .unwrap(),
            ]
            .into_iter()
            .flatten(),
            fee,
        )
        .unwrap();
        assert!(!closure.is_empty());
        assert!(closure.into_amounts().all(i128::is_negative))
    }
}
