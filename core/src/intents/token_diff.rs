use std::collections::BTreeMap;

use defuse_num_utils::CheckedMulDiv;
use impl_tools::autoimpl;
use near_sdk::{near, AccountIdRef};

use crate::{
    engine::State,
    fees::Pips,
    tokens::{TokenAmounts, TokenId},
    DefuseError, Result,
};

use super::ExecutableIntent;

pub type TokenDeltas = TokenAmounts<BTreeMap<TokenId, i128>>;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[autoimpl(Deref using self.diff)]
#[autoimpl(DerefMut using self.diff)]
pub struct TokenDiff {
    pub diff: TokenDeltas,
}

impl ExecutableIntent for TokenDiff {
    fn execute_intent<S>(self, signer_id: &AccountIdRef, state: &mut S) -> Result<()>
    where
        S: State,
    {
        if self.diff.is_empty() {
            return Err(DefuseError::ZeroAmount);
        }

        let fees = state.fee();
        let fee_collector = state.fee_collector().into_owned();

        for (token_id, delta) in self.diff {
            if delta == 0 {
                return Err(DefuseError::ZeroAmount);
            }

            state
                .internal_add_delta(signer_id.to_owned(), token_id.clone(), delta)
                .ok_or(DefuseError::BalanceOverflow)?;

            let amount = delta.unsigned_abs();
            let fee: i128 = Self::token_fee(&token_id, amount, fees)
                .fee_ceil(amount)
                .try_into()
                .unwrap_or_else(|_| unreachable!());

            state
                .internal_add_delta(fee_collector.clone(), token_id.clone(), fee)
                .ok_or(DefuseError::BalanceOverflow)?;
        }

        Ok(())
    }
}

impl TokenDiff {
    /// Returns [`TokenDiff`] closure to successfully execute `self`
    /// assuming given `fee`
    #[inline]
    pub fn closure(self, fee: Pips) -> Option<Self> {
        Self::closure_deltas(self.diff.into_inner(), fee).map(|diff| Self { diff })
    }

    /// Returns [`TokenDiff`] closure to successfully execute given set
    /// of distinct [`TokenDiff`] assuming given `fee`
    #[inline]
    pub fn closure_many(diffs: impl IntoIterator<Item = Self>, fee: Pips) -> Option<Self> {
        Self::closure_deltas(diffs.into_iter().flat_map(|d| d.diff.into_inner()), fee)
            .map(|diff| Self { diff })
    }

    /// Returns closure for deltas that should be given in a single
    /// [`TokenDiff`] to successfully execute given set of distinct `deltas`
    /// assuming given `fee`
    pub fn closure_deltas(
        deltas: impl IntoIterator<Item = (TokenId, i128)>,
        fee: Pips,
    ) -> Option<TokenDeltas> {
        deltas
            .into_iter()
            // accumulate deltas with fees
            .try_fold(TokenDeltas::default(), |mut deltas, (token_id, delta)| {
                let delta_in_with_fee = Self::token_delta_in_with_fee(&token_id, delta, fee)?;
                deltas
                    .add_delta(token_id, delta_in_with_fee)
                    .map(|_| deltas)
            })?
            .into_inner()
            .into_iter()
            // calculate closure
            .try_fold(TokenDeltas::default(), |mut deltas, (token_id, delta)| {
                let delta_out = Self::token_delta_out(&token_id, delta, fee)?;
                deltas.add_delta(token_id, delta_out).map(|_| deltas)
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
            Pips::MAX.as_pips().into(),
            i128::from(Pips::MAX.as_pips()) - delta.signum() * i128::from(fee.as_pips()),
        )
    }
}

// #[derive(Debug, Default)]
// struct DeltaTransferEventsBuilder {
//     withdraw_token_ids: Vec<defuse_nep245::TokenId>,
//     withdraw_amounts: Vec<U128>,
//     deposit_token_ids: Vec<defuse_nep245::TokenId>,
//     deposit_amounts: Vec<U128>,
// }

// impl DeltaTransferEventsBuilder {
//     #[inline]
//     pub fn push_delta(&mut self, token_id: &TokenId, delta: i128) {
//         let (token_ids, amounts) = if delta.is_negative() {
//             (&mut self.withdraw_token_ids, &mut self.withdraw_amounts)
//         } else {
//             (&mut self.deposit_token_ids, &mut self.deposit_amounts)
//         };
//         token_ids.push(token_id.to_string());
//         amounts.push(U128(delta.unsigned_abs()));
//     }

//     pub fn make<'a>(
//         &'a self,
//         signer: &'a AccountIdRef,
//         collector: &'a AccountIdRef,
//     ) -> MtEvent<'a> {
//         MtEvent::MtTransfer(Cow::Owned(
//             [
//                 (!self.withdraw_amounts.is_empty()).then(|| {
//                     Self::make_event(
//                         signer,
//                         collector,
//                         &self.withdraw_token_ids,
//                         &self.withdraw_amounts,
//                     )
//                 }),
//                 (!self.deposit_amounts.is_empty()).then(|| {
//                     Self::make_event(
//                         collector,
//                         signer,
//                         &self.deposit_token_ids,
//                         &self.deposit_amounts,
//                     )
//                 }),
//             ]
//             .into_iter()
//             .flatten()
//             .collect(),
//         ))
//     }

//     #[inline]
//     fn make_event<'a>(
//         old_owner_id: &'a AccountIdRef,
//         new_owner_id: &'a AccountIdRef,
//         token_ids: &'a [defuse_nep245::TokenId],
//         amounts: &'a [U128],
//     ) -> MtTransferEvent<'a> {
//         MtTransferEvent {
//             authorized_id: None,
//             old_owner_id: old_owner_id.into(),
//             new_owner_id: new_owner_id.into(),
//             token_ids: token_ids.into(),
//             amounts: amounts.into(),
//             memo: Some("token_diff".into()),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

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
                + i128::try_from(
                    TokenDiff::token_fee(&token_id, delta_abs, fee).fee_ceil(delta_abs)
                )
                .unwrap()
                + closure
                + i128::try_from(
                    TokenDiff::token_fee(&token_id, closure_abs, fee).fee_ceil(closure_abs)
                )
                .unwrap(),
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
                        TokenDeltas::default()
                            .with_add_deltas([(t1.clone(), d1), (t2.clone(), d2)])
                            .unwrap(),
                        TokenDeltas::default()
                            .with_add_deltas([(t3.clone(), d3)])
                            .unwrap(),
                    ]
                    .into_iter()
                    .flatten(),
                    fee
                )
                .unwrap(),
                TokenDeltas::default()
                    .with_add_deltas([
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
                TokenDeltas::default()
                    .with_add_deltas([(t1.clone(), -100), (t2.clone(), 200)])
                    .unwrap(),
                TokenDeltas::default()
                    .with_add_deltas([(t2.clone(), -200), (t3.clone(), 300)])
                    .unwrap(),
                TokenDeltas::default()
                    .with_add_deltas([(t3.clone(), -300), (t1.clone(), 101)])
                    .unwrap(),
            ]
            .into_iter()
            .flatten(),
            fee,
        )
        .unwrap();
        assert!(!closure.is_empty());
        assert!(closure.into_inner().into_values().all(i128::is_negative))
    }
}
