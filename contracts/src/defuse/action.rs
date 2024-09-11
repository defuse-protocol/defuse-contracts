use near_sdk::near;

use super::verify::diff::SignedDiffs;

#[derive(Debug)]
#[near(serializers = [borsh, json])]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    ApplySignedDiffs { diffs: SignedDiffs },
    // TODO: withdraw
}
