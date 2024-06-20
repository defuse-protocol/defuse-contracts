use std::borrow::Borrow;

use defuse_contracts::{
    intents::mpc::{
        Account, Action, Intent, IntentID, MpcIntentContract, MpcIntentError, RegisteredIntent,
    },
    utils::Mutex,
};
use near_contract_standards::non_fungible_token::core::{ext_nft_core, NonFungibleTokenReceiver};
use near_sdk::{
    borsh::BorshSerialize,
    env, near,
    store::{lookup_map::Entry, LookupMap},
    AccountId, AccountIdRef, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum Prefix {
    Intents,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct MpcIntentContractImpl {
    // TODO: store intent statuses along with intents
    intents: LookupMap<IntentID, Mutex<RegisteredIntent>>,
}

#[near]
impl MpcIntentContractImpl {
    #[init]
    pub fn new() -> Self {
        Self {
            intents: LookupMap::new(Prefix::Intents),
        }
    }
}

#[near]
impl MpcIntentContract for MpcIntentContractImpl {
    fn rollback_intent(&mut self, id: IntentID) -> Promise {
        todo!()
    }
}

#[near]
impl NonFungibleTokenReceiver for MpcIntentContractImpl {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> PromiseOrValue<bool> {
        // TODO: check predeccessor id to be subaccount of `.defuse`
        let account_shard = env::predecessor_account_id();
        // TODO
        // assert!(
        //     account_shard.is_sub_account_of(AccountIdRef::new_or_panic(
        //         // TODO: store TLA in state?
        //         "defuse"
        //     )),
        //     "only calls from Defuse Account Shards are allowed"
        // );

        let action = Action::decode(msg).expect("decode Action");
        let received = Account {
            account_shard,
            derivation_path: token_id,
        };

        match action {
            Action::Create { id, intent } => {
                // assert_eq!(
                //     (account_shard, token_id),
                //     (intent.send.account_shard, intent.send.derivation_path),
                //     "(account_shard, derivation_path) mismatch"
                // );
                self.create_intent(id, received, intent)
                    .expect("failed to create intent")
            }
            Action::Fulfill {
                id,
                recipient,
                memo,
                msg,
            } => self.execute_intent(id, received, recipient.unwrap_or(sender_id), memo, msg),
        }
    }
}

impl MpcIntentContractImpl {
    fn create_intent(
        &mut self,
        id: IntentID,
        received: Account,
        intent: Intent,
    ) -> Result<PromiseOrValue<bool>, MpcIntentError> {
        match self.intents.entry(id) {
            Entry::Occupied(entry) => {
                return Err(MpcIntentError::AlreadyExists(entry.key().clone()))
            }
            Entry::Vacant(entry) => {
                entry.insert(
                    RegisteredIntent {
                        send: received,
                        intent,
                    }
                    .into(),
                );
            }
        };
        Ok(PromiseOrValue::Value(false))
    }

    fn intent_mut<Q: ?Sized>(&mut self, id: &Q) -> Option<&mut Mutex<RegisteredIntent>>
    where
        IntentID: Borrow<Q>,
        Q: BorshSerialize + ToOwned<Owned = IntentID>,
    {
        self.intents.get_mut(id)
    }

    fn execute_intent(
        &mut self,
        id: IntentID,
        received: Account,
        recipient: AccountId,
        memo: Option<String>,
        msg: Option<String>,
    ) -> PromiseOrValue<bool> {
        let intent = self
            .intent_mut(&id)
            .ok_or_else(|| MpcIntentError::NotFound(id.to_owned()))
            .unwrap()
            .lock_mut()
            .expect("locked");

        match &intent.intent {
            Intent::Barter(barter) => {
                assert_eq!(received, barter.receive, "received wrong account");

                let p = ext_nft_core::ext(received.account_shard)
                    .with_attached_deposit(NearToken::from_yoctonear(1));
                if let Some(msg) = barter.msg.clone() {
                    p.nft_transfer_call(
                        barter.recepient.clone(),
                        barter.receive.derivation_path.clone(),
                        None,
                        barter.memo.clone(),
                        msg,
                    )
                } else {
                    p.nft_transfer(
                        barter.recepient.clone(),
                        barter.receive.derivation_path.clone(),
                        None,
                        barter.memo.clone(),
                    )
                }
                .and({
                    let p = ext_nft_core::ext(intent.send.account_shard.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(1));
                    if let Some(msg) = msg {
                        p.nft_transfer_call(
                            recipient,
                            intent.send.derivation_path.clone(),
                            None,
                            memo,
                            msg,
                        )
                    } else {
                        p.nft_transfer(recipient, intent.send.derivation_path.clone(), None, memo)
                    }
                })
                .then(Self::ext(env::current_account_id()).cleanup_intent(id))
            }
        }
        .into()
    }
}

#[near]
impl MpcIntentContractImpl {
    // TODO: refund according to storage management
    #[private]
    // #[handle_result]
    // TODO: return Some(true)?
    pub fn cleanup_intent(&mut self, id: IntentID) {
        match self.intents.entry(id) {
            Entry::Vacant(_) => env::panic_str("intent doesn't exist"),
            Entry::Occupied(mut entry) => {
                assert!(entry.get_mut().unlock(), "not locked");
                entry.remove();
            }
        }
    }
}
