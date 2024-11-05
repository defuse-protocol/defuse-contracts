pub use self::{deadline::*, error::*, gas::*, lock::*};

pub mod access_keys;
pub mod bitmap;
pub mod borsh;
pub mod cache;
pub mod cleanup;
mod deadline;
mod error;
pub mod fees;
mod gas;
pub mod integer;
mod lock;
pub mod prefix;
pub mod serde;

#[macro_export]
macro_rules! method_name {
    ($ty:ident::$method:ident) => {{
        // check that method exists
        const _: *const () = $ty::$method as *const ();
        stringify!($method)
    }};
}
