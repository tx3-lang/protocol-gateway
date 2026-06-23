# snek.fun

tx3 protocol for [snek.fun](https://snek.fun/) — a fair-launch token launchpad on Cardano. Tokens launch on a bonding curve at the same starting valuation; prices rise with demand, and once a token reaches a 69 000 ADA market cap it graduates and its liquidity migrates to Splash DEX with the LP burned. See [docs.snek.fun](https://docs.snek.fun/getting-started/introduction) for the protocol overview.

> **Status: WIP** — This tx3 off-chain implementation is under testing and may not cover all edge cases yet.

## Overview

Each token launched on snek.fun has its own bonding curve pool (v1 protocol). Trades follow a two-step model:

1. **User submits an order** — sends ADA (buy) or tokens (sell) to a per-user order validator address with an inline datum describing the desired swap.
2. **Permitted executor processes the order** — the snek.fun batcher consumes the order UTxO together with the bonding curve pool UTxO, fills the trade, and returns funds to the user.

This tx3 implements the **user-facing flow only**: placing orders, cancelling orders, and launching new tokens. Pool spends (the actual fills) require the permitted-executor signature plus dynamic input/output indices and are out of scope.

## Transactions

| Transaction | Description |
|---|---|
| `place_buy_order` | Deposit ADA at the order validator; the batcher will return tokens. |
| `place_sell_order` | Deposit tokens at the order validator; the batcher will return ADA. |
| `cancel_order` | Spend a user's own order UTxO back to their wallet (`Cancel` redeemer). |
| `launch_token` | Mint a new token (1B supply), seed its bonding curve pool, deposit metadata, and pay the launch fee. |

## Important Considerations

- **Order submissions are script-free:** `place_buy_order` and `place_sell_order` are plain payments to the order validator address with an inline datum — no validator runs. The script only fires when the batcher fills (or the user cancels).
- **Per-user order address:** `OrderScript` is `(payment = order validator script, stake = user's own stake key)`. Because the stake side changes per user it cannot live in the env — callers must pass the concrete bech32 per call as the `orderscript` party.
- **Per-launch token policy:** `launch_token` requires a parameterised minting policy (new policy id for every token). The caller must apply the seed outref to the on-disk template off-chain and pass both the resulting `token_policy` (script hash) and `token_script` (CBOR) per call.
- **Address enums (tx3 0.23):** `OrderDatum.owner_addr` is modelled with proper Cardano-shaped enums — `PlutusAddress(Credential, OptionalStake)`, `Credential::PubKey/Script`, `StakingCred::Hash`, `OptionalStake::Some/None` — built inline by the shared `owner_address(pkh, stake_key)` fn. (Earlier versions used a chain of single-field structs to dodge an arg-encoding bug; self-describing args (#343) fixed it.)
- **Derived amounts (tx3 0.23):** the verifiable escrow/supply figures are now computed on-chain via `+`/`-` (#339/#340): buy escrow = `ada_input + executor_fee + min_order_out_ada`, sell escrow = `executor_fee + min_order_out_ada`, curve supply = `token_emission - initial_buy_tokens`. Independent min-ADA/fee/curve figures (`pool_seed_ada`, `launch_fee_ada`, `ada_cap_thresh_for_pool`, …) are not derivable on-chain and stay caller params.
- **Direction marker fixed at 1:** `OrderAmount.direction` is hardcoded to `1` (exact-input variant). The `direction=0` "buy-with-output" mode is not implemented.
- **Pool spends not implemented:** Buy/sell fills against the bonding curve are done by the snek.fun batcher and require the permitted-executor signature + dynamic input/output indices — they cannot be modelled in tx3 today.

## Caller Preparation

### `place_buy_order`

- `orderscript` party — bech32 of `(order validator script + user's stake key)`. Must be assembled per user.
- `user_pkh`, `user_stake_key` — raw 28-byte hashes from the user's wallet, also embedded in the datum's nested owner address.
- `token_policy`, `token_name` — the token information the user is buying (queried from snek.fun for that pool).
- `ada_input` — lovelace the batcher may spend on tokens (caller chooses based on slippage tolerance).
- `deadline_ms` — unix millis after which the batcher rejects the order.

The escrow total (`ada_input + executor_fee + min_order_out_ada`) and the ADA asset id (`""`/`""`) are now computed on-chain, so callers no longer pass `total_escrow_ada` or an `empty_bytes` placeholder.

### `place_sell_order`

Same as `place_buy_order` plus:
- `token_amount` — token quantity being sold.
- The escrow ADA (`executor_fee + min_order_out_ada` = 2 600 000 lovelace) is derived on-chain, so callers don't pass it.

### `cancel_order`

- `order_utxo: UtxoRef` — the order UTxO to reclaim. The user must also have a separate pure-ADA UTxO available for fees and collateral.

### `launch_token`

This is the most parameter-heavy transaction. The caller must precompute everything off-chain:

| Parameter | Source |
|---|---|
| `seed_utxo`, `seed_tx`, `seed_idx` | A spendable UTxO from the creator's wallet — used to parameterise the token policy and as the input to the pool NFT mint redeemer. |
| `token_policy`, `token_script` | Apply the on-disk token mint template (`investigation/scripts/token_mint.v3.template.cbor.hex`) to the seed outref. The resulting blake2b-224 hash is `token_policy`; the applied CBOR is `token_script`. |
| `pool_nft_name` | 32-byte hash the pool NFT policy expects, derived from `(seed_outref_tx, seed_outref_idx)`. |
| `metadata_nft_name` | Asset name of the metadata NFT (raw bytes). |
| `meta: LaunchMetadata` | Metadata record (#343): `{ ticker, logo_cid, description, launch_type, socials: { twitter, discord, telegram, website }, version }` — bytes fields hex-encoded. Passed as a self-describing struct arg. |
| `author: PoolAuthor` | Creator address parts `{ pkh, skh }`, recorded in the metadata datum. Self-describing struct arg. |
| `pool_witness_pkh` | Pool authority PKH, recorded in the pool datum. |
| `ada_cap_thresh_for_pool` | Per-pool graduation threshold (close to `18_188_400_000` ± per-launch jitter). |
| `launch_fee_ada` | Fee paid to the snek.fun collector (observed = 1 825 000). |
| `metadata_min_ada`, `creator_min_ada`, `pool_seed_ada` | Min-ADA values for the metadata, creator, and pool outputs. |
| `initial_buy_tokens` | The creator's "dev buy". The bonding-curve seed (`token_emission - initial_buy_tokens`) is derived on-chain, so `curve_tokens_remaining` is no longer a param. |

The `meta` and `author` records are passed in the tagged self-describing form, e.g.:

```json
"author": { "struct": { "constructor": 0, "fields": [
  { "bytes": "<creator_pkh>" }, { "bytes": "<creator_stake_key>" } ] } }
```

See [`invoke-args/launch_token.json`](invoke-args/launch_token.json) for the full `meta` example.

Token policy application snippet (Python):

```python
import hashlib
tpl = bytes.fromhex(open('investigation/scripts/token_mint.v3.template.cbor.hex').read())
seed_tx  = bytes.fromhex('<your seed tx hash>')   # 32 bytes
seed_idx = 0                                       # 0..23
applied  = tpl[:475] + seed_tx + bytes([seed_idx]) + tpl[475+33:]
policy   = hashlib.blake2b(b'\x03' + applied, digest_size=28).hexdigest()
# feed `applied.hex()` as token_script and `policy` as token_policy
```

## tx3 Limitations

> Updated for tx3 0.23: the former "no `*`/`/`" and "no nested enum variants" limitations are **resolved**
> (see `protocols_tx3/TX3-0.23-UPGRADE.md`). What remains blocked:

- **Exponential bonding curve:** `price = a·exp(b·sold)` needs `exp`/fixed-point math, which tx3 lacks. The verifiable linear/subtractive splits are inlined, but `ada_input` / `initial_buy_tokens` (the curve-derived figures) stay caller-computed off-chain.
- **Per-launch parameterised script:** `token_script` is passed in as raw CBOR via `cardano::plutus_witness` because the policy is parameterised per launch (new policy id every time) — tx3 cannot apply parameters to a script template.
- **Pool spend not modellable:** Filling an order requires the permitted-executor signature plus dynamic input/output ordering against the pool UTxO; this can't be expressed in tx3 today and is left to the snek.fun batcher.
- **Resolver version:** the inlined arithmetic needs a resolver ≥0.22 and the `meta`/`author` self-describing params need ≥0.23. The mainnet TRP is ≥0.23 — all four txs resolve live (verified 2026-06-23: `place_buy_order` datum is byte-identical to the on-chain Poppy order; `launch_token`'s `PoolDatum` matches the real Poppy launch in 7/9 fields).

## CBOR Verification

All four user-facing transactions have been compiled, built, and compared against real on-chain transactions (Poppy launch, Poppy buy/sell, live LAIKA cancel target). Datums match byte-for-byte; `launch_token` outputs match Poppy's launch shape with byte-identical fee and pool-seed lines. See [`invoke-args/comparison-report.md`](invoke-args/comparison-report.md) for the full field-by-field breakdown and reference tx hashes.

To reproduce:

```bash
cd protocols_tx3/snek-fun

trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/place_buy_order.json
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/place_sell_order.json
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/cancel_order.json
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/launch_token.json
```

## Smart Contracts

- Order validator: PlutusV2 (reference script at `e2ed9e953ebf98ca701fc93588d73cb9769f87b9d13712474f566a0743963e8b#0`)
- Bonding curve & pool NFT policy: PlutusV2 (stable across all pools)
- Per-launch token mint: PlutusV3 (parameterised, new policy id per launch)
- Source: closed; on-chain shapes reverse-engineered — see [`investigation/snek-fun-research.md`](investigation/snek-fun-research.md).
