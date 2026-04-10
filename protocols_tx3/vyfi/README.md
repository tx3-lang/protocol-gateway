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

## Important Considerations

- **No script execution on submission:** Order submissions are plain payments with inline datums — no validators are invoked. This means no collateral is needed.
- **Pool-specific order address:** The `OrderScript` party must be set to the correct order address for each pool. Pool data is available from the VyFi API.
- **Process fee:** VyFi batchers charge a process fee (currently 1.9 ADA) included in each order.
- **User credentials format:** The `user_creds` parameter is a 56-byte value: payment credential (28 bytes) concatenated with staking credential (28 bytes).
- **Batcher-side not implemented:** This tx3 does not cover batcher operations (consuming orders, updating pool state). Only user-facing order submission and cancellation.

## Caller Preparation

### All transactions

- `user_creds: Bytes` — 56-byte hex value: payment credential (28 bytes) + staking credential (28 bytes). Must be constructed from the user's wallet address by extracting the raw credential hashes and concatenating them.
- `OrderScript` party address — Must be set to the correct **pool-specific** order address. Query the VyFi API (`/lp?networkId=1`) for pool data including the order address.

### `swap_b_to_a` / `remove_liquidity`

- `order_ada: Int` — The ADA amount to include in the order UTxO (min UTxO + process fee). Must account for the tokens being sent.

### `add_liquidity`

- `desired_lp: Int` — The desired LP token amount. Must be calculated based on current pool ratio.

### Pool data (query VyFi API before any transaction)

- Pool order address (for `OrderScript` party)
- Token policy ID and asset name (for token parameters)
- Current pool ratio (for calculating swap amounts, LP tokens)
- Process fee: currently 1.9 ADA

## tx3 Limitations

No significant tx3 limitations were encountered. Order submissions are plain payments with inline datums — no script execution, no complex redeemers. The two-step model means all validator logic runs on the batcher side (not implemented in this tx3).

## Smart Contracts

- Pool data source: VyFi API (`/lp?networkId=1`)
