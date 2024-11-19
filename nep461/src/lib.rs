pub trait SignedMessageNep {
    const NEP_NUMBER: u32;
}

pub trait OffchainMessage: SignedMessageNep {
    const OFFCHAIN_PREFIX_TAG: u32 = (1u32 << 31) + Self::NEP_NUMBER;
}
impl<T> OffchainMessage for T where T: SignedMessageNep {}

pub trait OnchainMessage: SignedMessageNep {
    const OFFCHAIN_PREFIX_TAG: u32 = (1u32 << 30) + Self::NEP_NUMBER;
}
impl<T> OnchainMessage for T where T: SignedMessageNep {}
