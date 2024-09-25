use std::{fmt::Display, str::FromStr};

use near_sdk::serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Helper type to implement `#[derive(Serialize, Deserialize)]`,
/// as `#[near_bindgen]` doesn't support `#[serde(...)]` attributes on method arguments
#[serde_as]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(crate = "::near_sdk::serde")]
pub struct DisplayFromStr<T>(#[serde_as(as = "serde_with::DisplayFromStr")] pub T)
where
    T: Display + FromStr<Err: Display>;

#[cfg(all(feature = "abi", not(target_arch = "wasm32")))]
mod abi {
    use super::*;

    use near_sdk::schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    impl<T> JsonSchema for DisplayFromStr<T>
    where
        T: Display + FromStr<Err: Display>,
    {
        fn schema_name() -> String {
            String::schema_name()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            String::json_schema(gen)
        }

        fn is_referenceable() -> bool {
            false
        }
    }
}
