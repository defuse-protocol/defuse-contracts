use impl_tools::autoimpl;
use near_sdk::near;

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
                    let delta_in_with_fee = Self::token_delta_in_with_fee(&token_id, delta, fee)
                        .ok_or(DefuseError::BalanceOverflow)?;
                    deltas.add(token_id, delta_in_with_fee).map(|_| deltas)
                },
            )?
            .into_iter()
            // calculate closure
            .try_fold(TokenAmounts::default(), |mut deltas, (token_id, delta)| {
                let delta_out = Self::token_delta_out(&token_id, delta, fee)
                    .ok_or(DefuseError::BalanceOverflow)?;
                deltas.add(token_id, delta_out).map(|_| deltas)
            })
    }

    /// Returns closure for delta that should be given in a single
    /// [`TokenDiff`] to successfully execute [`TokenDiff`] with given
    /// `delta` on the same token assuming given `fee`.
    ///
    /// Formula: `⌊-(delta + ⌈|delta| * fee⌉) / (1 - fee * sign(delta))⌋`
    #[inline]
    pub fn closure_delta(token_id: &TokenId, delta: i128, fee: Pips) -> Option<i128> {
        Self::token_delta_out(
            token_id,
            Self::token_delta_in_with_fee(token_id, delta, fee)?,
            fee,
        )
    }

    #[inline]
    pub const fn token_fee(token_id: &TokenId, amount: u128, fee: Pips) -> Pips {
        match token_id {
            TokenId::Nep141(_) => {}
            TokenId::Nep245(_, _) if amount > 1 => {}
            // do not take fees on NFTs and MTs with |delta| <= 1
            _ => return Pips::ZERO,
        }
        fee
    }

    #[inline]
    pub fn token_delta_in_with_fee(token_id: &TokenId, delta: i128, fee: Pips) -> Option<i128> {
        Self::delta_in_with_fee(delta, Self::token_fee(token_id, delta.unsigned_abs(), fee))
    }

    /// Returns `delta + ⌈|delta| * fee⌉`
    #[inline]
    fn delta_in_with_fee(delta: i128, fee: Pips) -> Option<i128> {
        delta.checked_add_unsigned(fee.fee_ceil(delta.unsigned_abs()))
    }

    #[inline]
    pub fn token_delta_out(token_id: &TokenId, delta: i128, fee: Pips) -> Option<i128> {
        Self::delta_out(delta, Self::token_fee(token_id, delta.unsigned_abs(), fee))
    }

    /// Returns `⌊-delta / (1 - fee * sign(delta))⌋`
    #[inline]
    fn delta_out(delta: i128, fee: Pips) -> Option<i128> {
        delta.checked_neg()?.checked_mul_div_euclid(
            Pips::MAX.as_pips() as i128,
            Pips::MAX.as_pips() as i128 - delta.signum() * fee.as_pips() as i128,
        )
    }
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
        #[values(
            (TokenId::Nep141("ft.near".parse().unwrap()), 1_000_000), (TokenId::Nep141("ft.near".parse().unwrap()), -1_000_000),
            (TokenId::Nep171("nft.near".parse().unwrap(), "1".to_string()), 1), 
            (TokenId::Nep171("nft.near".parse().unwrap(), "1".to_string()), -1),
            (TokenId::Nep245("mt.near".parse().unwrap(), "ft1".to_string()), 1_000_000),
            (TokenId::Nep245("mt.near".parse().unwrap(), "ft1".to_string()), -1_000_000),
            (TokenId::Nep245("mt.near".parse().unwrap(), "nft1".to_string()), 1), 
            (TokenId::Nep245("mt.near".parse().unwrap(), "nft1".to_string()), -1),
        )]
        token_delta: (TokenId, i128),
        #[values(
            Pips::ZERO,
            Pips::ONE_PIP,
            Pips::ONE_BIP,
            Pips::ONE_PERCENT,
            Pips::ONE_PERCENT * 50,
        )]
        fee: Pips,
    ) {
        let (token_id, delta) = token_delta;
        let closure = TokenDiff::closure_delta(&token_id, delta, fee).unwrap();

        let delta_abs = delta.unsigned_abs();
        let closure_abs = closure.unsigned_abs();
        assert_eq!(
            delta
                + TokenDiff::token_fee(&token_id, delta_abs, fee).fee_ceil(delta_abs) as i128
                + closure
                + TokenDiff::token_fee(&token_id, closure_abs, fee).fee_ceil(closure_abs) as i128,
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
                        (t1.clone(), TokenDiff::closure_delta(&t1, d1, fee).unwrap()),
                        (t2.clone(), TokenDiff::closure_delta(&t2, d2, fee).unwrap()),
                        (t3.clone(), TokenDiff::closure_delta(&t3, d3, fee).unwrap()),
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
