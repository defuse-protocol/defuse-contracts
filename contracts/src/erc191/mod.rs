use near_sdk::env;

/// See [personal_sign](https://github.com/ethereum/ercs/blob/master/ERCS/erc-191.md#version-0x45-e)
#[inline]
pub fn sign_hash(data: &[u8]) -> [u8; 32] {
    env::keccak256_array(
        // 0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>
        &[
            format!("\x19Ethereum Signed Message:\n{}", data.len()).as_bytes(),
            data,
        ]
        .concat(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;

    #[test]
    fn test_hash() {
        assert_eq!(
            sign_hash(b"Please sign this message to confirm your identity."),
            hex!("c21712258067502aad461ea687c066dfebd518e90f5b57d4cc04f5b3eb34f00e")
        );
    }
}
