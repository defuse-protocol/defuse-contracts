use core::{
    fmt::{self, Display},
    str::FromStr,
};

use derive_more::derive::From;
use near_sdk::serde::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeFromStr, SerializeDisplay};

/// Helper type to implement `#[derive(Serialize, Deserialize)]`,
/// as `#[near_bindgen]` doesn't support `#[serde(...)]` attributes on method arguments
#[derive(Debug, Clone, Copy, Default, SerializeDisplay, DeserializeFromStr, From)]
pub struct DisplayFromStr<T>(pub T);

impl<T> DisplayFromStr<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Display for DisplayFromStr<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> FromStr for DisplayFromStr<T>
where
    T: FromStr,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        T::from_str(s).map(Self)
    }
}

/// Helper type to implement `#[derive(Serialize, Deserialize)]`,
/// as `#[near_bindgen]` doesn't support `#[serde(...)]` attributes on method arguments
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    serde_as(schemars = true)
)]
#[cfg_attr(
    not(all(feature = "abi", not(target_arch = "wasm32"))),
    serde_as(schemars = false)
)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, From)]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    derive(::near_sdk::schemars::JsonSchema)
)]
#[serde(
    crate = "::near_sdk::serde",
    bound(serialize = "T: AsRef<[u8]>", deserialize = "T: TryFrom<Vec<u8>>")
)]
#[cfg_attr(
    all(feature = "abi", not(target_arch = "wasm32")),
    schemars(crate = "::near_sdk::schemars", transparent)
)]
pub struct Base64<T>(#[serde_as(as = "super::base64::Base64")] pub T);

impl<T> Base64<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

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
