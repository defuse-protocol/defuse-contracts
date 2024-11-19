mod cache;
mod gas;
mod lock;
mod panic;
mod prefix;

pub use self::{cache::*, gas::*, lock::*, panic::*, prefix::*};

#[macro_export]
macro_rules! method_name {
    ($ty:ident::$method:ident) => {{
        // check that method exists
        const _: *const () = $ty::$method as *const ();
        stringify!($method)
    }};
}
