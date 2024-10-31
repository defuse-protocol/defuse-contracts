use near_sdk::env;

#[inline]
pub fn storage_delta(mut f: impl FnMut()) -> i64 {
    let initial = env::storage_usage();
    f();
    let current = env::storage_usage() as i64;
    current.saturating_sub_unsigned(initial)
}
