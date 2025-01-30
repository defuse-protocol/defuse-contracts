use defuse_crypto::PublicKey;
pub use near_account_id::*;
use near_sdk::env;

/// Reserved TLA for P256 (a.k.a. secp256r1, prime256v1) signatures.
///
/// Subaccounts are named as hex of last 20 bytes of SHA-256 hash of
/// public key in uncompressed form, i.e. concatenated x || y with
/// no leading SEC-1 tag byte.
///
/// Example: `b883e61c704ce539ad137daffe6559c3fe42d201.p256`
pub const P256_TLA: &AccountIdRef = AccountIdRef::new_or_panic("p256");

/// In order to keep compatibility with all existing standards within
/// Near ecosystem (e.g. NEP-245), we need our implicit account_ids
/// to be fully backwards-compatible with Near's implicit AccountId.
///
/// This backwards-compatibility is achieved by "reserving" TLAs
/// and other account schemes that, if introduced, would allow
/// singing transactions using correspnding curve type by default.
pub trait Implicit: sealed::Sealed {
    fn is_implicit(&self) -> bool;

    /// Returns whether this AccountId should have given public key by default.
    ///
    /// There can be multiple account-creation schemes, which provide different
    /// implicit account IDs for same public key.
    fn is_implicit_for(&self, public_key: &PublicKey) -> bool;

    /// Try to extract default public key from an implicit account ID
    fn to_default_public_key(&self) -> Option<PublicKey>;
}

impl Implicit for AccountIdRef {
    fn is_implicit(&self) -> bool {
        match self.get_account_type() {
            AccountType::NearImplicitAccount | AccountType::EthImplicitAccount => true,
            AccountType::NamedAccount => {
                // See docs for [`P256_TLA`]
                if self.is_sub_account_of(P256_TLA) {
                    return true;
                }
                // other account-creation schemes can be introduced later...
                false
            }
        }
    }

    fn is_implicit_for(&self, pk: &PublicKey) -> bool {
        match (self.get_account_type(), pk) {
            (AccountType::NearImplicitAccount, PublicKey::Ed25519(pk)) => {
                // https://docs.near.org/concepts/protocol/account-id#implicit-address
                return self == hex::encode(pk);
            }
            // WARN: some legacy Eth Implicit Acounts were created on Near
            // before web3-wallets feature was introduced. For these accounts,
            // it's important to forbid usage of implicit public keys. This can
            // be enforced from client-side.
            (AccountType::EthImplicitAccount, PublicKey::Secp256k1(pk)) => {
                // https://ethereum.org/en/developers/docs/accounts/#account-creation
                return self == format!("0x{}", hex::encode(&env::keccak256_array(pk)[12..32]));
            }
            (AccountType::NamedAccount, pk) => match pk {
                PublicKey::P256(pk) => {
                    if self.is_sub_account_of(P256_TLA)
                        && self
                            == format!("{}.{P256_TLA}", hex::encode(&env::sha256_array(pk)[12..32]))
                    {
                        return true;
                    }
                    // other account-creation schemes can be introduced later...
                }
                _ => {}
            },
            (_, _) => {}
        }
        false
    }

    fn to_default_public_key(&self) -> Option<PublicKey> {
        if matches!(self.get_account_type(), AccountType::NearImplicitAccount) {
            return hex::decode(self.as_str())
                .ok()
                .and_then(|bytes| bytes.try_into().ok())
                .map(PublicKey::Ed25519);
        }
        None
    }
}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for AccountIdRef {}
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn implicit_eth() {
        const ACCOUNT_ID: &AccountIdRef =
            AccountIdRef::new_or_panic("0x388c818ca8b9251b393131c08a736a67ccb19297");

        assert!(ACCOUNT_ID.is_implicit());
        assert_eq!(ACCOUNT_ID.to_default_public_key(), None);
    }

    #[test]
    fn implicit_near() {
        const ACCOUNT_ID: &AccountIdRef = AccountIdRef::new_or_panic(
            "b029600066b79e7040b9dca00c548e8d124966bd931c758e00a49c8fa625c9b8",
        );
        const PUBLIC_KEY: PublicKey = PublicKey::Ed25519(hex!(
            "b029600066b79e7040b9dca00c548e8d124966bd931c758e00a49c8fa625c9b8"
        ));

        assert!(ACCOUNT_ID.is_implicit());
        assert!(ACCOUNT_ID.is_implicit_for(&PUBLIC_KEY));
        assert_eq!(ACCOUNT_ID.to_default_public_key(), Some(PUBLIC_KEY));
    }

    #[test]
    fn implicit_p256_tla() {
        const ACCOUNT_ID: &AccountIdRef =
            AccountIdRef::new_or_panic("b883e61c704ce539ad137daffe6559c3fe42d201.p256");

        assert!(ACCOUNT_ID.is_implicit());
        assert!(ACCOUNT_ID.is_implicit_for(&"p256:2V8Np9vGqLiwVZ8qmMmpkxU7CTRqje4WtwFeLimSwuuyF1rddQK5fELiMgxUnYbVjbZHCNnGc6fAe4JeDcVxgj3Q".parse().unwrap()));
        assert_eq!(ACCOUNT_ID.to_default_public_key(), None);
    }

    #[test]
    fn named() {
        const ACCOUNT_ID: &AccountIdRef = AccountIdRef::new_or_panic("test.near");

        assert!(!ACCOUNT_ID.is_implicit());
        assert!(!ACCOUNT_ID.is_implicit_for(&"p256:2V8Np9vGqLiwVZ8qmMmpkxU7CTRqje4WtwFeLimSwuuyF1rddQK5fELiMgxUnYbVjbZHCNnGc6fAe4JeDcVxgj3Q".parse().unwrap()));
        assert_eq!(ACCOUNT_ID.to_default_public_key(), None);
    }

    #[test]
    fn tla() {
        const ACCOUNT_ID: &AccountIdRef = AccountIdRef::new_or_panic("near");

        assert!(!ACCOUNT_ID.is_implicit());
        assert!(!ACCOUNT_ID.is_implicit_for(&"p256:2V8Np9vGqLiwVZ8qmMmpkxU7CTRqje4WtwFeLimSwuuyF1rddQK5fELiMgxUnYbVjbZHCNnGc6fAe4JeDcVxgj3Q".parse().unwrap()));
        assert_eq!(ACCOUNT_ID.to_default_public_key(), None);
    }
}
