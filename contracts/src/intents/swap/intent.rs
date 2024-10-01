use core::cmp;

use near_sdk::{near, AccountId};

use crate::utils::Deadline;

use super::{events::Dip2Event, AssetWithAccount, LostAsset, SwapIntentError};

pub type IntentId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
#[serde(rename_all = "snake_case")]
pub struct SwapIntent {
    /// Provided asset as an input.
    pub asset_in: AssetWithAccount,

    /// Desired asset as an output.
    // TODO: multiple inputs, outputs, e.g. for storage deposit
    pub asset_out: AssetWithAccount,

    /// Optional lockup period for [`asset_in`] when initiator cannot rollback
    /// the intent.  
    /// NOTE: MUST come before [`expiration`]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockup_until: Option<Deadline>,

    /// Deadline to execute the swap.  
    /// NOTE: intent can still be rollbacked at any time unless
    /// [`lockup_until`] is specified.
    pub expiration: Deadline,

    /// Current status of the intent
    #[serde(flatten)]
    pub status: SwapIntentStatus,

    /// Lost asset in case the intent has already been executed/rollbacked
    /// but we failed to transfer an asset to recipient/initiator.
    /// This can happen due to recipient/initiator is not registered
    /// on the target asset contract or does not have enough storage
    /// deposited according to Storage Management standard (NEP-145).
    /// Anyone can call `lost_found(intent_id)` to retry the transfer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lost: Option<LostAsset>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referral: Option<AccountId>,
}

impl SwapIntent {
    #[must_use]
    #[inline]
    pub const fn initiator(&self) -> &AccountId {
        self.asset_in.initiator()
    }

    #[must_use]
    #[inline]
    pub const fn is_available(&self) -> bool {
        self.status.is_available()
    }

    #[must_use]
    #[inline]
    pub fn has_expired(&self) -> bool {
        self.expiration.has_expired()
    }

    #[must_use]
    #[inline]
    pub fn is_locked_up(&self) -> bool {
        !self.lockup_until.map_or(true, Deadline::has_expired)
    }

    #[inline]
    pub fn set_executed(&mut self, id: &IntentId, proof: Option<String>, lost: Option<LostAsset>) {
        self.status = SwapIntentStatus::Executed { proof };
        Dip2Event::Executed(id).emit();

        if let Some(lost) = lost {
            self.set_lost(id, lost);
        }
    }

    #[inline]
    pub fn set_rolled_back(&mut self, id: &IntentId, lost: bool) {
        self.status = SwapIntentStatus::RolledBack;
        Dip2Event::RolledBack(id).emit();

        if lost {
            self.set_lost(
                id,
                LostAsset::AssetIn {
                    recipient: self.asset_in.account(),
                },
            );
        }
    }

    #[inline]
    pub fn set_lost(&mut self, id: &IntentId, lost: LostAsset) {
        Dip2Event::Lost {
            intent_id: id,
            asset: &lost,
        }
        .emit();
        self.lost = Some(lost);
    }

    #[must_use]
    #[inline]
    pub fn lost_asset(&self) -> Option<AssetWithAccount> {
        match self.lost.clone()? {
            LostAsset::AssetIn { recipient } => self.asset_in.with_account(recipient),
            LostAsset::AssetOut => Some(self.asset_out.clone()),
        }
    }

    #[inline]
    pub fn lost_found(&mut self, id: &IntentId) {
        if self.lost.take().is_some() {
            Dip2Event::Found(id).emit();
        }
    }

    // Check that lockup_until <= expiration
    fn validate_lockup(&self) -> Result<(), SwapIntentError> {
        if let Some(lockup_until) = self.lockup_until {
            if matches!(
                lockup_until.partial_cmp(&self.expiration),
                Some(cmp::Ordering::Greater),
            ) {
                return Err(SwapIntentError::LockupAfterExpiration);
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SwapIntentError> {
        // prevent from swapping from/to zero amount
        if self.asset_in.is_zero_amount() || self.asset_out.is_zero_amount() {
            return Err(SwapIntentError::ZeroAmount);
        }

        // prevent from creating already expired intents
        if self.has_expired() || self.lockup_until.map_or(false, Deadline::has_expired) {
            return Err(SwapIntentError::Expired);
        }

        self.validate_lockup()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum SwapIntentStatus {
    /// Available for execution.
    #[default]
    Available,
    /// Executed.
    Executed {
        /// Optional proof for cross-chain assets.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        proof: Option<String>,
    },
    /// Rolled back.
    RolledBack,
}

impl SwapIntentStatus {
    #[must_use]
    #[inline]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    #[must_use]
    #[inline]
    pub const fn is_executed(&self) -> bool {
        matches!(self, Self::Executed { .. })
    }
}
