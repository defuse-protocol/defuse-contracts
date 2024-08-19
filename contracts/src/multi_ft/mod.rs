use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId, Promise, PromiseOrValue};

pub use types::Approval;

mod types;

#[ext_contract(ext_multi_ft)]
pub trait MultiFungibleToken {
    /// Simple transfer. Transfer a given `token_id` from current owner to
    /// `receiver_id`.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security purposes
    /// * Caller must have greater than or equal to the `amount` being requested
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    ///
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token
    /// * `token_id`: the token to transfer
    /// * `amount`: the number of tokens to transfer, wrapped in quotes and treated
    ///    like a string, although the number will be stored as an unsigned integer
    ///    with 128 bits.
    /// * `approval` (optional): is a tuple of [`owner_id`,`approval_id`].
    ///   `owner_id` is the valid Near account that owns the tokens.
    ///   `approval_id` is the expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: AccountId,
        amount: U128,
        approval: Option<Approval>,
        memo: Option<String>,
    );

    /// Simple batch transfer. Transfer a given `token_ids` from current owner to
    /// `receiver_id`.
    ///
    /// Requirements
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security purposes
    /// * Caller must have greater than or equal to the `amounts` being requested for the given `token_ids`
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    /// * Contract MUST panic if called with the length of `token_ids` not equal to `amounts` is not equal
    /// * Contract MUST panic if `approval_ids` is not `null` and does not equal the length of `token_ids`
    ///
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token
    /// * `token_ids`: the tokens to transfer
    /// * `amounts`: the number of tokens to transfer, wrapped in quotes and treated
    ///    like an array of strings, although the numbers will be stored as an array of unsigned integer
    ///    with 128 bits.
    /// * `approvals` (optional): is an array of expected `approval` per `token_ids`.
    ///    If a `token_id` does not have a corresponding `approval` then the entry in the array
    ///    must be marked null.
    ///   `approval` is a tuple of [`owner_id`,`approval_id`].
    ///   `owner_id` is the valid Near account that owns the tokens.
    ///   `approval_id` is the expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer
    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Approval>>,
        memo: Option<String>,
    );

    /// Transfer token and call a method on a receiver contract. A successful
    /// workflow will end in a success execution outcome to the callback on the MT
    /// contract at the method `mt_resolve_transfer`.
    ///
    /// You can think of this as being similar to attaching native NEAR tokens to a
    /// function call. It allows you to attach any Multi Token, token in a call to a
    /// receiver contract.
    ///
    /// Requirements:
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
    ///   purposes
    /// * Caller must have greater than or equal to the `amount` being requested
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * The receiving contract must implement `mt_on_transfer` according to the
    ///   standard. If it does not, MT contract's `mt_resolve_transfer` MUST deal
    ///   with the resulting failed cross-contract call and roll back the transfer.
    /// * Contract MUST implement the behavior described in `mt_resolve_transfer`
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    ///
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token.
    /// * `token_id`: the token to send.
    /// * `amount`: the number of tokens to transfer, wrapped in quotes and treated
    ///    like a string, although the number will be stored as an unsigned integer
    ///    with 128 bits.
    /// * `owner_id`: the valid NEAR account that owns the token
    /// * `approval` (optional): is a tuple of [`owner_id`,`approval_id`].
    ///   `owner_id` is the valid Near account that owns the tokens.
    ///   `approval_id` is the expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer.
    /// * `msg`: specifies information needed by the receiving contract in
    ///    order to properly handle the transfer. Can indicate both a function to
    ///    call and the parameters to pass to that function.

    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: AccountId,
        amount: U128,
        approval: Option<Vec<Approval>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// Transfer tokens and call a method on a receiver contract. A successful
    /// workflow will end in a success execution outcome to the callback on the MT
    /// contract at the method `mt_resolve_transfer`.
    //
    /// You can think of this as being similar to attaching native NEAR tokens to a
    /// function call. It allows you to attach any Multi Token, token in a call to a
    /// receiver contract.
    //
    /// Requirements:
    /// * Caller of the method must attach a deposit of 1 yoctoⓃ for security
    ///   purposes
    /// * Caller must have greater than or equal to the `amount` being requested
    /// * Contract MUST panic if called by someone other than token owner or,
    ///   if using Approval Management, one of the approved accounts
    /// * The receiving contract must implement `mt_on_transfer` according to the
    ///   standard. If it does not, MT contract's `mt_resolve_transfer` MUST deal
    ///   with the resulting failed cross-contract call and roll back the transfer.
    /// * Contract MUST implement the behavior described in `mt_resolve_transfer`
    /// * `approval_id` is for use with Approval Management extension, see
    ///   that document for full explanation.
    /// * If using Approval Management, contract MUST nullify approved accounts on
    ///   successful transfer.
    /// * Contract MUST panic if called with the length of `token_ids` not equal to `amounts` is not equal
    /// * Contract MUST panic if `approval_ids` is not `null` and does not equal the length of `token_ids`
    //
    /// Arguments:
    /// * `receiver_id`: the valid NEAR account receiving the token.
    /// * `token_ids`: the tokens to transfer
    /// * `amounts`: the number of tokens to transfer, wrapped in quotes and treated
    ///    like an array of string, although the numbers will be stored as an array of
    ///    unsigned integer with 128 bits.
    /// * `approvals` (optional): is an array of expected `approval` per `token_ids`.
    ///    If a `token_id` does not have a corresponding `approval` then the entry in the array
    ///    must be marked null.
    ///    `approval` is a tuple of [`owner_id`,`approval_id`].
    ///   `owner_id` is the valid Near account that owns the tokens.
    ///   `approval_id` is the expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    /// * `memo` (optional): for use cases that may benefit from indexing or
    ///    providing information for a transfer.
    /// * `msg`: specifies information needed by the receiving contract in
    ///    order to properly handle the transfer. Can indicate both a function to
    ///    call and the parameters to pass to that function.

    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Approval>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>>;

    /// Finalize an `mt_transfer_call` or `mt_batch_transfer_call` chain of cross-contract calls. Generically
    /// referred to as `mt_transfer_call` as it applies to `mt_batch_transfer_call` as well.
    ///
    /// The `mt_transfer_call` process:
    ///
    /// 1. Sender calls `mt_transfer_call` on MT contract
    /// 2. MT contract transfers token from sender to receiver
    /// 3. MT contract calls `mt_on_transfer` on receiver contract
    /// 4+. [receiver contract may make other cross-contract calls]
    /// N. MT contract resolves promise chain with `mt_resolve_transfer`, and may
    ///    transfer token back to sender
    ///
    /// Requirements:
    /// * Contract MUST forbid calls to this function by any account except self
    /// * If promise chain failed, contract MUST revert token transfer
    /// * If promise chain resolves with `true`, contract MUST return token to
    ///   `sender_id`
    ///
    /// Arguments:
    /// * `sender_id`: the sender of `mt_transfer_call`
    /// * `receiver_id`: the `receiver_id` argument given to `mt_transfer_call`
    /// * `token_ids`: the `token_ids` argument given to `mt_transfer_call`
    /// * `amounts`: the `token_ids` argument given to `mt_transfer_call`
    /// * `approvals (optional)`: if using Approval Management, contract MUST provide
    ///   set of original approvals in this argument, and restore the
    ///   approved accounts in case of revert.
    ///   `approvals` is an array of expected `approval_list` per `token_ids`.
    ///   If a `token_id` does not have a corresponding `approvals_list` then the entry in the
    ///   array must be marked null.
    ///   `approvals_list` is an array of triplets of [`owner_id`,`approval_id`,`amount`].
    ///   `owner_id` is the valid Near account that owns the tokens.
    ///   `approval_id` is the expected approval ID. A number smaller than
    ///    2^53, and therefore representable as JSON. See Approval Management
    ///    standard for full explanation.
    ///   `amount`: the number of tokens to transfer, wrapped in quotes and treated
    ///    like a string, although the number will be stored as an unsigned integer
    ///    with 128 bits.
    ///
    ///
    ///
    /// Returns total amount spent by the `receiver_id`, corresponding to the `token_id`.
    /// The amounts returned, though wrapped in quotes and treated like strings,
    /// the numbers will be stored as an unsigned integer with 128 bits.
    /// Example: if sender_id calls `mt_transfer_call({ "amounts": ["100"], token_ids: ["55"], receiver_id: "games" })`,
    /// but `receiver_id` only uses 80, `mt_on_transfer` will resolve with `["20"]`, and `mt_resolve_transfer`
    /// will return `["80"]`.
    fn mt_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_ids: Vec<AccountId>,
        receiver_id: AccountId,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<(AccountId, U128, u64)>>,
    ) -> Vec<U128>;

    /// Returns the balance of an account for the given `token_id`.
    /// The balance though wrapped in quotes and treated like a string,
    /// the number will be stored as an unsigned integer with 128 bits.
    /// Arguments:
    /// * `account_id`: the NEAR account that owns the token.
    /// * `token_id`: the token to retrieve the balance from
    fn mt_balance_of(&self, account_id: AccountId, token_id: AccountId) -> U128;

    /// Returns the balances of an account for the given `token_ids`.
    /// The balances though wrapped in quotes and treated like strings,
    /// the numbers will be stored as an unsigned integer with 128 bits.
    /// Arguments:
    /// * `account_id`: the NEAR account that owns the tokens.
    /// * `token_ids`: the tokens to retrieve the balance from
    fn mt_batch_balance_of(&self, account_id: AccountId, token_ids: Vec<AccountId>) -> Vec<U128>;

    /// Returns the token supply with the given `token_id` or `None` if no such token exists.
    /// The supply though wrapped in quotes and treated like a string, the number will be stored
    /// as an unsigned integer with 128 bits.
    fn mt_supply(&self, token_id: AccountId) -> Option<U128>;

    /// Returns the token supplies with the given `token_ids`, a string value is returned or `None`
    /// if no such token exists. The supplies though wrapped in quotes and treated like strings,
    /// the numbers will be stored as an unsigned integer with 128 bits.
    fn mt_batch_supply(&self, token_ids: Vec<AccountId>) -> Vec<Option<U128>>;
}

#[ext_contract(ext_multi_ft_receiver)]
pub trait MultiFungibleTokenReceiver {
    /// Take some action after receiving a multi token
    ///
    /// Requirements:
    /// * Contract MUST restrict calls to this function to a set of whitelisted
    ///   contracts
    /// * Contract MUST panic if `token_ids` length does not equal `amounts`
    ///   length
    /// * Contract MUST panic if `previous_owner_ids` length does not equal `token_ids`
    ///   length
    ///
    /// Arguments:
    /// * `sender_id`: the sender of `mt_transfer_call`
    /// * `previous_owner_ids`: the account that owned the tokens prior to it being
    ///   transferred to this contract, which can differ from `sender_id` if using
    ///   Approval Management extension
    /// * `token_ids`: the `token_ids` argument given to `mt_transfer_call`
    /// * `amounts`: the `token_ids` argument given to `mt_transfer_call`
    /// * `msg`: information necessary for this contract to know how to process the
    ///   request. This may include method names and/or arguments.
    ///
    /// Returns the number of unused tokens in string form. For instance, if `amounts`
    /// is `["10"]` but only 9 are needed, it will return `["1"]`. The amounts returned,
    /// though wrapped in quotes and treated like strings, the numbers will be stored as
    /// an unsigned integer with 128 bits.
    fn mt_on_transfer(
        &self,
        sender_id: AccountId,
        previous_owner_ids: Vec<AccountId>,
        token_ids: Vec<AccountId>,
        amounts: Vec<U128>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>>;
}
