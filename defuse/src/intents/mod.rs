mod fees;
pub mod runtime;
mod token_diff;
mod tokens;

use defuse_contracts::defuse::{
    intents::{DefuseIntents, SignedIntentExecutor},
    message::SignedDefuseMessage,
    Result,
};
use near_sdk::near;
use runtime::IntentExecutor;

use crate::{DefuseImpl, DefuseImplExt};

use self::runtime::Runtime;

#[near]
impl SignedIntentExecutor for DefuseImpl {
    #[handle_result]
    fn execute_signed_intents(
        &mut self,
        signed: Vec<SignedDefuseMessage<DefuseIntents>>,
    ) -> Result<()> {
        #[cfg(feature = "beta")]
        crate::beta::beta_access!(self);

        let mut rt = Runtime::new(&self.fees, &mut self.total_supplies);

        for signed in signed {
            let (signer_id, signer, intents) = self.accounts.verify_signed_message(signed)?;

            rt.execute_intent(&signer_id, signer, intents)?;
            // TODO: log intent hash?
        }

        rt.finalize(&mut self.accounts)
    }
}
