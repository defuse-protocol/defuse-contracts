use defuse_contracts::defuse::{fees::Fees, intents::Intent, Result};
use near_sdk::AccountId;

use crate::accounts::{Account, Accounts};

use super::{
    fees::FeesTracker,
    tokens::{TokenTracker, TransferTracker},
};

pub trait IntentExecutor<T> {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: T,
        referral: Option<&AccountId>,
    ) -> Result<()>;
}

pub struct Runtime {
    pub(super) fees: FeesTracker,
    pub(super) transfers: TransferTracker,
    pub(super) tokens: TokenTracker,
}

impl Runtime {
    #[inline]
    pub fn new(fees: Fees) -> Self {
        Self {
            fees: FeesTracker::new(fees),
            transfers: TransferTracker::default(),
            tokens: TokenTracker::default(),
        }
    }

    #[inline]
    pub fn finalize(mut self, accounts: &mut Accounts) -> Result<()> {
        self.transfers.finalize(accounts)?;
        self.fees.finalize(accounts, &mut self.tokens)?;
        self.tokens.finalize()
    }
}

impl IntentExecutor<Intent> for Runtime {
    fn execute_intent(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        intent: Intent,
        referral: Option<&AccountId>,
    ) -> Result<()> {
        match intent {
            Intent::AddPublicKey { public_key } => {
                account.add_public_key(account_id, public_key);
                Ok(())
            }
            Intent::RemovePublicKey { public_key } => {
                account.remove_public_key(account_id, &public_key);
                Ok(())
            }
            Intent::InvalidateNonces { nonces } => {
                for n in nonces {
                    let _ = account.commit_nonce(n);
                }
                Ok(())
            }
            Intent::TokenTransfer(intent) => self.transfers.transfer(
                &mut account.token_balances,
                intent.recipient_id,
                intent.tokens,
            ),
            Intent::TokensDiff(intent) => {
                self.execute_intent(account_id, account, intent, referral)
            }
            Intent::TokenWithdraw(intent) => todo!(),
        }
    }
}
