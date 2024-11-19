use std::borrow::Cow;

use near_sdk::{env, near};

use crate::crypto::Payload;

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone)]
pub struct Erc191Payload<'a>(pub Cow<'a, str>);

impl<'a> Payload for Erc191Payload<'a> {
    fn hash(&self) -> near_sdk::CryptoHash {
        let data = self.0.as_bytes();
        env::keccak256_array(
            // 0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>
            &[
                format!("\x19Ethereum Signed Message:\n{}", data.len()).as_bytes(),
                data,
            ]
            .concat(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;

    #[test]
    fn test_hash() {
        assert_eq!(
            Erc191Payload("Please sign this message to confirm your identity.".into()).hash(),
            hex!("c21712258067502aad461ea687c066dfebd518e90f5b57d4cc04f5b3eb34f00e")
        );
    }
}
