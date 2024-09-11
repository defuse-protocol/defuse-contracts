use near_sdk::{
    base64::{engine::general_purpose::STANDARD, Engine},
    borsh::{self, io, BorshDeserialize, BorshSerialize},
};

#[inline]
pub fn as_base64<T, W>(obj: &T, writer: &mut W) -> Result<(), io::Error>
where
    T: BorshSerialize,
    W: io::Write,
{
    let v = borsh::to_vec(obj)?;
    let s = STANDARD.encode(v);
    s.serialize(writer)
}

#[inline]
pub fn from_base64<T, R>(reader: &mut R) -> Result<T, io::Error>
where
    T: BorshDeserialize,
    R: io::Read,
{
    let s: String = borsh::BorshDeserialize::deserialize_reader(reader)?;
    let v = STANDARD.decode(s).map_err(io::Error::other)?;
    borsh::from_slice(&v)
}
