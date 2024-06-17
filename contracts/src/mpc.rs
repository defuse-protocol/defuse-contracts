use near_sdk::{ext_contract, PromiseOrValue};

#[ext_contract(ext_mpc)]
pub trait MpcRecovery {
    fn sign(
        &self,
        payload: Vec<u8>,
        path: &str,
        key_version: u32,
    ) -> PromiseOrValue<(String, String)>;
}
