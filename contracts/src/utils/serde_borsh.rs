use near_sdk::{
    borsh,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
};

pub fn deserialize_json<T, R>(reader: &mut R) -> borsh::io::Result<T>
where
    T: DeserializeOwned,
    R: borsh::io::Read,
{
    let v: Vec<u8> = borsh::BorshDeserialize::deserialize_reader(reader)?;
    serde_json::from_slice(&v).map_err(borsh::io::Error::other)
}

pub fn serialize_json<T, W>(obj: &T, writer: &mut W) -> borsh::io::Result<()>
where
    T: Serialize,
    W: borsh::io::Write,
{
    let v = serde_json::to_vec(obj).map_err(borsh::io::Error::other)?;
    borsh::BorshSerialize::serialize(&v, writer)
}
