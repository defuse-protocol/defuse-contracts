mod execute;
mod relayer;
mod simulate;
mod state;
mod tokens;

use defuse_core::{
    crypto::{Payload, SignedPayload},
    engine::{Engine, State, StateView},
    intents::{DefuseIntents, ExecutableIntent},
    payload::{multi::MultiPayload, DefusePayload, ExtractDefusePayload},
    DefuseError, Result,
};
use defuse_near_utils::{UnwrapOrPanic, CURRENT_ACCOUNT_ID};
use execute::ExecuteInspector;
use near_plugins::{pause, Pausable};
use near_sdk::near;
use simulate::SimulateInspector;

use crate::intents::{Intents, SimulationOutput, StateOutput, TokenDiffOutput};

use super::{Contract, ContractExt};

#[near]
impl Intents for Contract {
    #[pause(name = "intents")]
    #[inline]
    fn execute_intents(&mut self, intents: Vec<MultiPayload>) {
        let mut engine = Engine::new(self, ExecuteInspector::default());
        engine
            .execute_signed_intents::<_, DefuseIntents>(intents)
            .unwrap_or_panic();
        engine.finalize().unwrap_or_panic().into_event().emit();

        // TODO: finalize transfers & emit?
        // TODO: finalize?
    }

    #[pause(name = "intents")]
    #[inline]
    fn simulate_intents(&self, intents: Vec<MultiPayload>) -> SimulationOutput {
        let mut inspector = SimulateInspector::default();
        let mut engine = Engine::new(self.cached(), &mut inspector);
        engine
            .execute_signed_intents::<_, DefuseIntents>(intents)
            .unwrap_or_panic();
        SimulationOutput {
            intents_executed: inspector.intents_executed,
            min_deadline: inspector.min_deadline,
            token_diff: TokenDiffOutput { closure: todo!() },
            state: StateOutput { fee: self.fee() },
        }
    }
}

// impl Contract {
//     pub(crate) fn engine(&self) -> Engine<SimulationHandler<CachedState<&Self>>> {
//         Engine::new(SimulationHandler::new(self.cached()))
//     }

//     pub(crate) fn engine_mut(&mut self) -> Engine<&mut Self> {
//         Engine::new(self)
//     }
// }

// pub struct SimulationHandler<S: State> {
//     state: S,
//     min_deadline: Deadline,
//     intents_executed: Vec<AccountEvent<'static, IntentExecutedEvent>>,
// }

// impl<S> SimulationHandler<S>
// where
//     S: State,
// {
//     pub fn new(state: S) -> Self {
//         Self {
//             state,
//             min_deadline: Deadline::MAX,
//             intents_executed: Vec::new(),
//         }
//     }
// }

// impl<S> Handler for SimulationHandler<S>
// where
//     S: State,
// {
//     #[inline]
//     fn on_intent_deadline(&mut self, deadline: Deadline) {
//         self.min_deadline = self.min_deadline.min(deadline);
//     }

//     fn emit<'a>(&mut self, event: impl Into<Event<'a>>) {
//         let Event::Defuse(DefuseEvent::IntentsExecuted(intents_executed)) = event.into() else {
//             return;
//         };

//         self.intents_executed.extend(
//             intents_executed
//                 .into_iter()
//                 .map(|event| event.clone().into_owned()),
//         );
//     }

//     #[inline]
//     fn on_mt_batch_transfer(
//         &mut self,
//         _sender_id: &AccountIdRef,
//         _transfer: MtBatchTransfer,
//     ) -> Result<()> {
//         Ok(())
//     }

//     #[inline]
//     fn on_ft_withdraw(&mut self, _sender_id: &AccountIdRef, _withdraw: FtWithdraw) -> Result<()> {
//         Ok(())
//     }

//     #[inline]
//     fn on_nft_withdraw(&mut self, _sender_id: &AccountIdRef, _withdraw: NftWithdraw) -> Result<()> {
//         Ok(())
//     }

//     #[inline]
//     fn on_mt_withdraw(&mut self, _sender_id: &AccountIdRef, _withdraw: MtWithdraw) -> Result<()> {
//         Ok(())
//     }
// }

// impl<S> StateView for SimulationHandler<S>
// where
//     S: State,
// {
//     #[inline]
//     fn verifying_contract(&self) -> Cow<'_, AccountIdRef> {
//         self.state.verifying_contract()
//     }

//     #[inline]
//     fn wnear_id(&self) -> Cow<'_, AccountIdRef> {
//         self.state.wnear_id()
//     }

//     #[inline]
//     fn fee(&self) -> Pips {
//         self.state.fee()
//     }

//     #[inline]
//     fn fee_collector(&self) -> Cow<'_, AccountIdRef> {
//         self.state.fee_collector()
//     }

//     #[inline]
//     fn has_public_key(&self, account_id: &AccountIdRef, public_key: &PublicKey) -> bool {
//         self.state.has_public_key(account_id, public_key)
//     }

//     #[inline]
//     fn iter_public_keys(&self, account_id: &AccountIdRef) -> impl Iterator<Item = PublicKey> + '_ {
//         self.state.iter_public_keys(account_id)
//     }

//     #[inline]
//     fn is_nonce_used(&self, account_id: &AccountIdRef, nonce: Nonce) -> bool {
//         self.state.is_nonce_used(account_id, nonce)
//     }

//     #[inline]
//     fn balance_of(&self, account_id: &AccountIdRef, token_id: &TokenId) -> u128 {
//         self.state.balance_of(account_id, token_id)
//     }
// }

// impl<S> State for SimulationHandler<S>
// where
//     S: State,
// {
//     #[must_use]
//     #[inline]
//     fn add_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
//         self.state.add_public_key(account_id, public_key)
//     }

//     #[must_use]
//     #[inline]
//     fn remove_public_key(&mut self, account_id: AccountId, public_key: PublicKey) -> bool {
//         self.state.remove_public_key(account_id, public_key)
//     }

//     #[must_use]
//     #[inline]
//     fn commit_nonce(&mut self, account_id: AccountId, nonce: Nonce) -> bool {
//         self.state.commit_nonce(account_id, nonce)
//     }

//     #[must_use]
//     #[inline]
//     fn internal_deposit(
//         &mut self,
//         owner_id: AccountId,
//         token_id: TokenId,
//         amount: u128,
//     ) -> Option<u128> {
//         self.state.internal_deposit(owner_id, token_id, amount)
//     }

//     #[must_use]
//     #[inline]
//     fn internal_withdraw(
//         &mut self,
//         owner_id: AccountId,
//         token_id: TokenId,
//         amount: u128,
//     ) -> Option<u128> {
//         self.state.internal_withdraw(owner_id, token_id, amount)
//     }

//     #[inline]
//     fn internal_add_delta(
//         &mut self,
//         owner_id: AccountId,
//         token_id: TokenId,
//         delta: i128,
//     ) -> Option<u128> {
//         self.state.internal_add_delta(owner_id, token_id, delta)
//     }

//     #[must_use]
//     #[inline]
//     fn deposit(&mut self, owner_id: AccountId, token_id: TokenId, amount: u128) -> Option<u128> {
//         self.state.deposit(owner_id, token_id, amount)
//     }

//     #[must_use]
//     #[inline]
//     fn withdraw(&mut self, owner_id: AccountId, token_id: TokenId, amount: u128) -> Option<u128> {
//         self.state.withdraw(owner_id, token_id, amount)
//     }
// }
