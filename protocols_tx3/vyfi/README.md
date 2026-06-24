# VyFi DEX

tx3 protocol for [VyFi](https://vyfi.io/) — AMM DEX order submission on Cardano.

> **Status: WIP** — This tx3 off-chain implementation is under testing and may not cover all edge cases yet.

## Overview

VyFi is an AMM-based DEX on Cardano using a two-step order model:

1. **User submits order** — sends funds + datum to a pool-specific order script address (no scripts executed)
2. **Batcher processes order** — VyFi infrastructure consumes the order + pool UTxO and distributes results

This tx3 implements the **user-facing transactions only** (step 1). Each pool has its own order address, provided dynamically per call via the `OrderScript` party.

## Transactions

| Transaction | Description |
|---|---|
| `swap_a_to_b` | Swap ADA for a token |
| `swap_b_to_a` | Swap a token for ADA |
| `add_liquidity` | Deposit ADA + tokens into a liquidity pool |
| `remove_liquidity` | Withdraw liquidity by sending LP tokens |
| `cancel` | Reclaim a pending order (spends it at the order validator) |

## Important Considerations

- **No script execution on submission:** Order submissions are plain payments with inline datums — no validators are invoked. This means no collateral is needed. (`cancel` is the exception — it spends the order at the validator, so it does need collateral.)
- **Pool-specific order address:** The `OrderScript` party must be set to the correct order address for each pool. Pool data is available from the VyFi API.
- **Process fee:** VyFi batchers charge a process fee (currently 1.9 ADA) included in each order.
- **User credentials format:** The `user_creds` parameter is a 56-byte value: payment credential (28 bytes) concatenated with staking credential (28 bytes).
- **Batcher-side not implemented:** This tx3 does not cover batcher operations (consuming orders, updating pool state). Only user-facing order submission and cancellation.

## Caller Preparation

### All transactions

- `user_creds: Bytes` — 56-byte hex value: payment credential (28 bytes) + staking credential (28 bytes). Must be constructed from the user's wallet address by extracting the raw credential hashes and concatenating them.
- `OrderScript` party address — Must be set to the correct **pool-specific** order address. Query the VyFi API (`/lp?networkId=1`) for pool data including the order address.

### `swap_b_to_a` / `remove_liquidity`

- `order_min_ada: Int` — The min-UTxO ADA buffer to hold in the order UTxO. The
  transaction adds `process_fee` (env) on top, so the order ends up with
  `order_min_ada + process_fee` lovelace plus the tokens. On-chain this buffer is a
  frontend choice — `2_000_000` is the common value (→ 3.9 ADA order), but it varies
  (1.05M – 2.1M observed). It is **not** the protocol's computed `min_utxo`, so it is
  passed by the caller.

### `cancel`

- `order_utxo: UtxoRef` — the order to reclaim, at the pool's order address.
- `order_script: Bytes` — the pool's order validator (PlutusV1). Pass the
  doubly-wrapped CBOR (the bytes whose blake2b-224 with the `0x01` tag equals the
  order address' script hash — i.e. what Koios `script_info`/`tx_info.bytecode`
  returns, e.g. ADA/SNEK starts `590a8c…`). VyFi uses no reference scripts, so the
  script is attached inline as a witness.
- `since_slot` / `until_slot: Int` — validity bounds (invalid_before / invalid_after).
  Set `since_slot` to the current tip and `until_slot` to `tip + ~10800` (~3h), as the
  real cancel tx does. Requires collateral and the owner's signature.

### `add_liquidity`

- `desired_lp: Int` — The desired LP token amount. Must be calculated based on current pool ratio.

> Note on the bundled `invoke-args/remove_liquidity.json`: a real call sets
> `lp_policy`/`lp_name` to the **pool's LP token** (e.g. ADA/SNEK =
> `630e2929…`/`VyFi_ADA/SNEK_LP`). The bundled fixture instead uses **SNEK as a
> stand-in** so it resolves against the test wallet (which holds no LP token) — the LP
> token lives in the order's *value*, not its datum, so the generated datum is identical
> either way. Its `user_creds`/`min_out_ada`/`min_out_token` are the real values of the
> remove-liq order that tx `34a41f7f` cancelled, so the datum value-matches on-chain.

### Pool data (query VyFi API before any transaction)

- Pool order address (for `OrderScript` party)
- Token policy ID and asset name (for token parameters)
- Current pool ratio (for calculating swap amounts, LP tokens)
- Process fee: currently 1.9 ADA

## tx3 Limitations

The four order submissions are plain payments with inline datums — no script
execution. `cancel` is the one script-executing tx: it spends the order at the
order validator (PlutusV1, attached inline since VyFi uses no reference scripts)
with the `Cancel` redeemer `Constr(1, [])`, both verified against the real cancel
tx `34a41f7f`.

Still out of scope / blocked: the batcher-side fills (dynamic input/output indices),
live pool reserves (they live in the UTxO *value*, not the datum, so `datum_is` can't
read them), and pool discovery (a VyFi-API concern — the order address, token info
and `order_script` are fetched off-chain per pool).

## Smart Contracts

- Pool data source: VyFi API (`/lp?networkId=1`)
