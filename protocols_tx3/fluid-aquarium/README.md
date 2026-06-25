# Fluid Aquarium

tx3 protocol for [FluidTokens Aquarium](https://github.com/FluidTokens/ft-cardano-aquarium-sc) — babel fees and scheduled transactions on Cardano.

> **Status: WIP** — This tx3 off-chain implementation is under testing and may not cover all edge cases yet.

## Overview

Fluid Aquarium allows users to create "tanks" (UTxOs with ADA) that can be consumed by:

- **Babel fee bots:** Pay a user's transaction fees in exchange for native tokens at an oracle-determined rate
- **Aquarium nodes:** Execute scheduled transactions at a specified future time

Users can also stake FLDT tokens to register as Aquarium node operators.

## Transactions

| Transaction | Description |
|---|---|
| `create_babel_tank` | Create a tank for babel fee payments (ADA deposit + datum) |
| `create_scheduled_tank` | Create a tank for scheduled transaction execution |
| `withdraw_tank` | Reclaim ADA from an owned tank |
| `consume_oracle` | Batcher consumes part of a tank's ADA, paying the owner in tokens at oracle rate |
| `execute_scheduled` | Batcher executes a scheduled transaction after its execution time |
| `stake_fldt` | Stake FLDT tokens to become an Aquarium node operator |

## Important Considerations

- **Tank creation is script-free:** `create_babel_tank` and `create_scheduled_tank` are plain payments with inline datums — no validators execute.
- **Oracle validation:** `consume_oracle` uses a withdrawal-based oracle (0 ADA withdrawal with redeemer). The oracle redeemer currently supports Charli3 price feeds.
- **Withdrawal redeemer:** Fixed ([tx3#317](https://github.com/tx3-lang/tx3/pull/317)). The reward redeemer is now correctly compiled into the witness set.
- **Typed datums & redeemers (tx3 0.23):** All Plutus data is now built inline from typed params — the tank `TankDatum`, the scheduled-tx `PlutusAddress`, and the `StakerRedeemer` no longer require raw CBOR ([tx3#343](https://github.com/tx3-lang/tx3/pull/343), self-describing args). A shared `fn owner_address` builds the on-chain address shape.
- **Oracle datum not readable via `datum_is`:** The Charli3 oracle provider datum uses a Map-based CBOR format (`Constr(0, [Constr(2, [Map {...}])])`) which tx3 types cannot model (they require positional constructor fields). Oracle price/validity values must still be passed as individual params.
- **Reference inputs ordering:** `consume_oracle` relies on multiple reference inputs (oracle provider, oracle contract, staker, parameters NFT, tank ref script) — their indices in the transaction must match the redeemer fields.
- **Oracle signatures:** `consume_oracle` takes a real `signatures: List<OracleSignature>` param (empty for Charli3, which validates via the ref input; populated for the signature-bearing feed variants).

## Caller Preparation

Several values must be prepared off-chain before invoking transactions:

### `create_babel_tank` / `create_scheduled_tank`

- `owner_payment_hash` / `owner_stake_hash: Bytes` — The tank owner's payment + staking key hashes. The full `TankDatum` (owner/destination addresses, the empty-token `CardanoToken`s) is built inline from these — no more `tank_datum_cbor`.

### `consume_oracle`

This transaction requires extensive off-chain data:
- **Oracle price data:** `oracle_price`, `oracle_denominator`, `oracle_valid_from`, `oracle_valid_to`, `payment_token_policy`, `payment_token_name` — queried from the Charli3 oracle datum on-chain (Map-based → not readable via `datum_is`).
- **Reference input indices:** `input_tank_idx`, `oracle_idx`, `ref_params_idx`, `oracle_provider_idx`, `paying_token_idx` — must match the actual ordering of inputs in the built transaction.
- **Payment calculation:** `payment_ada`, `payment_token_qty`, `tank_return_ada`, `dest_ada` — computed from the oracle price and tank contents (the quantity also depends on the real tx fee, so it can't be inlined).
- **Signatures:** `signatures: List<OracleSignature>` — empty `[]` for Charli3 (a top-level list param is a plain JSON array, not a tagged `{"list":[]}`).

### `execute_scheduled`

- `batcher_payment_hash` / `batcher_stake_hash: Bytes` — The batcher's reward address parts. The `PlutusAddress` in the `ScheduledTx` redeemer is built inline from these — no more `batcher_addr_cbor`.
- `batcher_signer_hash: Bytes` — The batcher's **staking key hash** (not payment key). tx3's `signers` block only extracts payment keys.

### `stake_fldt`

- `owner_payment_hash` / `owner_stake_hash`, `signer_bot_payment_hash` / `signer_bot_stake_hash: Bytes`, plus `ref_index` / `output_staking` / `staking_inputs: Int` — the parts of the `StakerRedeemer`, built inline (no more `staker_redeemer_cbor`). The bot's stake hash equals the staker NFT name.
- `signer_hash: Bytes` — The user's **staking key hash** (same reason as above).

## tx3 Limitations

Several tx3 language limitations affected this protocol; most are now resolved. Fixed: field name shadowing ([tx3#316](https://github.com/tx3-lang/tx3/pull/316)), withdrawal redeemer generation ([tx3#317](https://github.com/tx3-lang/tx3/pull/317)), typed reference datum access ([tx3#318](https://github.com/tx3-lang/tx3/pull/318)), and — as of tx3 0.23 — **custom types / structs / lists as params** ([tx3#343](https://github.com/tx3-lang/tx3/pull/343)), which removed all the raw-CBOR workarounds. Remaining limitations: staking-key extraction (`signers{}` yields only the payment key), and the Charli3 Map-based oracle datum (positional-only tx3 can't read it). For the full list, see [investigation/tx3-limitations-aquarium.md](investigation/tx3-limitations-aquarium.md).

## Smart Contracts

- PlutusV3 (Aiken v1.1.9)
- Source: [FluidTokens/ft-cardano-aquarium-sc](https://github.com/FluidTokens/ft-cardano-aquarium-sc)
