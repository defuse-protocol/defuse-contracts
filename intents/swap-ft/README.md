# Intent contract for the Defuse project

## Considerations

### Lost & Found
The idea for lost&found assets comes from [ref.finance](https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/account_deposit.rs#L435):
In case of failure of asset transfer from intent contract to recipient, we can store additional map of lost&found balances.
The transfer can fail in following scenarios:
* Recipient was not registered on the contract of asset being transferred
* Recipient has insufficient storage deposit to receive an asset  
  This is relevant only for case with NFTs, since FT require an account to be registered only once, while NFTs require storage deposit for each tokenID owned.
* 