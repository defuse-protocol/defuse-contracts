# Swap Intent

## Assets

### Native

```json
{
    "type": "near",
    // sender in context of `asset_in`, receiver in context of `asset_out`
    "account": "user1.near",
    "native": "1000",
}
```

### Fungibnle Token

```json
{
    "type": "near",
    // sender in context of `asset_in`, receiver in context of `asset_out`
    "account": "user1.near",
    "ft": {
        "token": "dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near",
        "amount": "1000"
    }
}
```

### Non-fungible Token

```json
{
    "type": "near",
    // sender in context of `asset_in`, receiver in context of `asset_out`
    "account": "user1.near",
    "nft": {
        "collection": "accout-shard1.defuse.near",
        "token_id": "1"
    }
}
```

### Cross Chain

```json
{
    "type": "cross_chain",
    // sender in context of `asset_in`, receiver in context of `asset_out`
    "account": "0xbBa57FB6B4d95E61D8e057bd61Ba0Bb0a3802bfd",
    "oracle": "solver.near", 
    "asset": "eth:1:0xdAC17F958D2ee523a2206206994597C13D831ec7",
    "amount": "1000" // or token id for NFTs
}
```

## Actions

```json
{
    "type": "create", // or "execute"
    // other fields depending on "type"
}
```

### `create` Action

```json
{
    "id": "INTENT_ID",
    "asset_out": {},
    "lockup_until": "", // TODO optional
    "expiration": "",
    "referral": "referral.near" // optional
}
```

### `execute` Action

```json
{
    "id": "INTENT_ID",
    "receiver": "solver.near" // TODO: other chains
}
```

## Sending assets with actions

### Native

Send tx to `SWAP_INTENT_ID` (e.g. `esufed.near`) calling method `native_action` with following arguments:
```json
{
    "action": <ACTION>
}
```

Attached NEAR will be treated as `asset_in`

### Ft

Send tx to `token_id` calling method `ft_transfer_call` with following arguments:

```json
{
    "receiver_id": "SWAP_INTENT_ID", // e.g. "esufed.near"
    "amount": "1000",
    "memo": "<optional>",
    "msg": "<ACTION serialized to JSON>"
}
```

### Cross Chain

For now, solver acts as `oracle`, so it can call 

Send tx to `SWAP_INTENT_ID` (e.g. `esufed.near`) calling method `on_cross_chain_transfer` with following arguments:

```json
{
    "sender": "0x4DF4eB62FB50C1053aD7A1cc313a4BC1299e5981", // sender on foreign chain // TODO: remove or optional?
    "asset": "eth:1:0xdAC17F958D2ee523a2206206994597C13D831ec7",
    "amount": "1000",
    "extra": { // TODO
        "tx_hash": "0x1233",
    },
    "msg": "<ACTION serialized to JSON>"
}
```

#### Cross-chain asset format

* EVM: `eth:{decimal chain_id}:{ "native" | "0x" .. <hex address> }`
    * ETH (Ethereum Mainnet): `eth:1:native`
    * USDT (Ethereum Mainnet): `eth:1:0xdAC17F958D2ee523a2206206994597C13D831ec7`
    * USDC (Base mainnet): `eth:8453:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`
* TON: `ton:{decimal workchain_id}:{ "native" | <hex address> }`
    * TON (TON basechain): `ton:0:native`
    * USDT (TON basechain): `ton:0:b113a994b5024a16719f69139328eb759596c38a25f59028b146fecdc3621dfe`
* Solana: `solana:{ "mainnet" | "testnet" | "devnet" }:{ base58 address }`
    * SOL (Solana mainnet): `solana:mainnet:native`
    * USDT (Solana mainnet): `solana:mainnet:Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`
* NEAR: `near:{ "mainnet" | "testnet" }:{ "native" | <account_id> }`
    * NEAR (Near mainnet) : `near:mainnet:native`
    * USDT (Near mainnet): `near:mainnet:dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near`


## Rollback

Initiator of the intent (i.e. sender of `asset_in`) can send tx to `SWAP_INTENT_ID` (e.g. `esufed.near`) calling method `rollback_intent` with following arguments:

```json
{
    "id": "INTENT_ID"
}
``` 


## Lost&Found

If an asset transfer has failed during intent execution/rollback, there is a permission-less method `lost_found` to retry the transfer:

```json
{
    "id": "INTENT_ID"
}
```

## Get Intent

Call view method `get_swap_intent` on `SWAP_INTENT_ID` (i.e. `esufed.near`) with following arguments:

```json
{
    "id": "INTENT_ID"
}
```

It returns following struct:

```json
{
    "asset_in": {
        // sender
        "account": "user1.near",
        "native": "1000"
    },
    "asset_out": {
        // receiver
        "account": "user1.near",
        "ft": {
            "token": "dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near",
            "amount": "5000"
        },
    },
    "lockup_until": { "unix":  }, // TODO optional. indicates
    "exipiration": {}, // TODO
    "status": {},
    "referral": "referral.near", // optional
    "lost": 
    "locked": true // optional. if true, there is an ongoing operation taking place
}
```