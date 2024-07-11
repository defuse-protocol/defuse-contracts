use core::time::Duration;

use near_sdk::{env, near, AccountId};

use super::{Asset, LostAsset};

pub type IntentId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SwapIntentStatus {
    /// Available for execution.
    Available(SwapIntent),
    /// The intent has already been executed/rollbacked
    /// but we failed to transfer an asset to recipient/initiator.
    /// This can happen due to recipient/initiator is not registered
    /// on the target asset contract or does not have enough storage
    /// deposited according to Storage Management standard (NEP-145).
    /// Anyone can call `lost_found(intent_id)` to retry the transfer.
    Lost(LostAsset),
}

impl SwapIntentStatus {
    #[must_use]
    #[inline]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available(_))
    }

    #[must_use]
    #[inline]
    pub const fn as_available(&self) -> Option<&SwapIntent> {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::Available(swap) => Some(swap),
            _ => None,
        }
    }

    #[inline]
    pub fn as_available_mut(&mut self) -> Option<&mut SwapIntent> {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::Available(swap) => Some(swap),
            _ => None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_lost(&self) -> bool {
        matches!(self, Self::Lost(_))
    }

    #[must_use]
    #[inline]
    pub const fn as_lost(&self) -> Option<&LostAsset> {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::Lost(lost) => Some(lost),
            _ => None,
        }
    }

    #[inline]
    pub fn as_lost_mut(&mut self) -> Option<&mut LostAsset> {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::Lost(lost) => Some(lost),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[near(serializers = [borsh, json])]
pub struct SwapIntent {
    /// Initiator who created the intent.
    pub initiator: AccountId,
    /// Provided asset as an input.
    pub asset_in: Asset,
    /// Desired asset as an output.
    // TODO: in case of NFT, this only allows for simple "barter",
    // while in case of Defuse, the user doesn't know in advance which
    // account solver will use for this swap. Possible solutions for this issue:
    // * Accept whatever NFT from only whitelisted solvers
    // * Some kind of auction, where solvers "register" their willingness
    //   to close the intent and compete between each other over given
    //   set of properties. These properties of suggested addresses by solvers
    //   can be compared between each other either on-chain (by having
    //   light-client contracts for each chain) or by user front-ends:
    //   this info about offers can be presented to the user and user can
    //   accept the best one or chose between them.
    //   So, it will become 3-stage process. We need to thing about it properly
    pub asset_out: Asset,
    /// Where to send asset_out. By default: back to initiator.
    #[serde(default)]
    pub recipient: Option<AccountId>,
    /// Deadline to execute the swap.
    /// NOTE: intent can still be rollbacked at any time.
    pub expiration: Expiration,
}

impl SwapIntent {
    #[must_use]
    #[inline]
    pub fn has_expired(&self) -> bool {
        self.expiration.has_expired()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers=[borsh, json])]
#[serde(rename_all = "snake_case")]
pub enum Expiration {
    /// UNIX Timestamp in seconds
    Timestamp(u64),
    /// Block number
    BlockNumber(u64),
}

impl Expiration {
    #[must_use]
    #[inline]
    pub fn has_expired(self) -> bool {
        match self {
            Self::Timestamp(timestamp) => {
                env::block_timestamp_ms() > timestamp.saturating_mul(1000)
            }
            Self::BlockNumber(n) => env::block_height() > n,
        }
    }

    #[cfg(target_family = "wasm")]
    #[must_use]
    pub fn in_n_blocks(n: u64) -> Self {
        Self::BlockNumber(env::block_height().saturating_add(n))
    }

    #[cfg(target_family = "wasm")]
    #[must_use]
    pub fn timeout(timeout: Duration) -> Self {
        Self::Timestamp(
            env::block_timestamp_ms()
                .saturating_add(timeout.as_millis() as u64)
                .saturating_div(1000),
        )
    }

    #[cfg(not(target_family = "wasm"))]
    #[must_use]
    pub fn timeout(timeout: Duration) -> Self {
        Self::Timestamp(
            (std::time::SystemTime::now() + timeout)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }
}
