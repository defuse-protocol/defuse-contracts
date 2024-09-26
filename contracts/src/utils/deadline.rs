use core::{cmp, time::Duration};

use near_sdk::{env, near};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[near(serializers=[borsh, json])]
#[serde(rename_all = "snake_case")]
pub enum Deadline {
    /// UNIX Timestamp in seconds
    Timestamp(u64),
    /// Block number
    BlockNumber(u64),
}

#[cfg(feature = "unit-testing")]
impl Default for Deadline {
    #[inline]
    fn default() -> Self {
        Self::BlockNumber(u64::MAX)
    }
}

impl PartialOrd for Deadline {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Self::Timestamp(s), Self::Timestamp(other)) => s.partial_cmp(other),
            (Self::BlockNumber(n), Self::BlockNumber(other)) => n.partial_cmp(other),
            // no way to compare UNIX timestamp with block number
            _ => None,
        }
    }
}

impl Deadline {
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
