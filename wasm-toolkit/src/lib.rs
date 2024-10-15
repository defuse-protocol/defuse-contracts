use defuse_contracts::defuse::intents::DefuseIntents;
use defuse_contracts::defuse::payload::{DefusePayload, SignerPayload};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = parseAndSerializeSignerPayload)]
pub fn parse_and_serialize_singer_payload(serialized: String) -> Vec<u8> {
    let a: SignerPayload<DefuseIntents> = serde_json::from_str(&serialized).unwrap();
    borsh::to_vec(&a).unwrap()
}

#[wasm_bindgen(js_name = parseAndSerializeDefusePayload)]
pub fn parse_and_serialize_defuse_payload(serialized: String) -> Vec<u8> {
    let a: DefusePayload<DefuseIntents> = serde_json::from_str(&serialized).unwrap();
    borsh::to_vec(&a).unwrap()
}
