use defuse_contracts::intents::swap::SwapIntentAction;
use near_sdk::near;

use super::{SwapIntentContractImpl, SwapIntentContractImplExt};

/// THIS IS A HELPER STRUCT, IT IS NOT USED IN THE CONTRACT
#[near(serializers = [json])]
pub struct Abi(SwapIntentAction);

#[near]
impl SwapIntentContractImpl {
    /// THIS IS A HELPER FUNCTION, IT DOES NOT EXIST IN THE CONTRACT
    // HACK: `cargo near abi` looks at the symbol table in target executable
    // and exports ABI only for those types that were mentioned as arguments.
    pub fn abi(_abi: Abi) {}
}
