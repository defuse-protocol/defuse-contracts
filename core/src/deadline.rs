use core::{
    ops::{Add, AddAssign},
    time::Duration,
};

use chrono::{DateTime, Utc};
use near_sdk::near;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[near(serializers=[json])]
pub struct Deadline(
    #[cfg_attr(
        all(feature = "abi", not(target_arch = "wasm32")),
        schemars(with = "String", example = "Deadline::default")
    )]
    DateTime<Utc>,
);

impl Deadline {
    pub const MAX: Self = Self(DateTime::<Utc>::MAX_UTC);

    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub fn now() -> Self {
        Self(DateTime::from_timestamp_nanos(
            near_sdk::env::block_timestamp()
                .try_into()
                .unwrap_or_else(|_| unreachable!()),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    #[inline]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    #[must_use]
    #[inline]
    pub fn timeout(timeout: Duration) -> Self {
        Self::now() + timeout
    }

    #[must_use]
    #[inline]
    pub fn has_expired(self) -> bool {
        Self::now() > self
    }
}

impl Add<Duration> for Deadline {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<Duration> for Deadline {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs
    }
}

// #[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
// mod abi {
//     use super::*;

//     use near_sdk::schemars::{
//         gen::SchemaGenerator,
//         schema::{InstanceType, Schema, SchemaObject},
//         JsonSchema,
//     };

//     impl JsonSchema for Deadline {
//         fn schema_name() -> String {
//             String::new()
//         }

//         fn is_referenceable() -> bool {
//             false
//         }

//         fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
//             SchemaObject {
//                 instance_type: Some(InstanceType::String.into()),
//                 format: Some("date-time".to_string()),
//                 ..Default::default()
//             }
//             .into()
//         }
//     }
// }
