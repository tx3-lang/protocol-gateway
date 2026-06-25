# VyFi DEX — Comparison Report

## Reference Transactions (on-chain)

| Type | TX Hash | Pool |
|------|---------|------|
| Swap order submission (ADA→SNEK) | `db08967b2d6bd2b10d8c3ae072f9768fd24220639bbdc1fd95d1f54ea89f0b56` | ADA/SNEK |
| Swap order submission (SNEK→ADA) | `94eee7c4678c2ef90e65ad13cac0ffee6f6a5507397ffb7c31eecc7bdae1707b` | ADA/SNEK |
| Swap execution (batcher) | `bee950d8767ffae7b0ee6d92279e447cad5c7f569f3715cac4cc196d6e39d7b3` | ADA/SNEK |
| Order cancellation | `34a41f7f12bbb280a476d52f3802aabdd8575cfe96c277c3d7b866302a12d3b0` | ADA/SNEK |
| Add liquidity execution | `675c02603d413248e93727dd5516ebedede538fa5195b3bab57e9561bc2e4af9` | ADA/SNEK |
| Remove liquidity execution | `8da8e2e4dc74432573b857f2a035fc21ea3404b4a459f82191defd8c1799095c` | ADA/SNEK |
| Swap execution (ADA/USDA) | `75fea6c7dd233384d13880f278a3666cb3e0fa5520f86aa997082bbf56ab582b` | ADA/USDA |

## Generated Transactions

| TX | Status | Notes |
|----|--------|-------|
| swap_a_to_b | OK | Datum: Constr(0, [Bytes(56), Constr(3, [min_out])]) |
| swap_b_to_a | OK | Datum: Constr(0, [Bytes(56), Constr(4, [min_out])]) |
| add_liquidity | OK | Datum: Constr(0, [Bytes(56), Constr(0, [desired_lp])]) |
| remove_liquidity | OK | Datum: Constr(0, [Bytes(56), Constr(1, [Constr(0, [min_ada, min_token])])]) |
| cancel | OK | Spends order @ order script, redeemer Constr(1, []), PlutusV1 inline witness, validity [since, until] |

## Test Wallet

- Address: `addr1q93k6rgprz5fxwkpvl2vgjq4pwejth400f8aldz2m3lj7khrnd05p259l0qjrf396am6wahv5895ey35y62fexta3q5q3cc3k8`
- Payment cred: `636d0d0118a8933ac167d4c448150bb325deaf7a4fdfb44adc7f2f5a`
- Stake cred: `e39b5f40aa85fbc121a625d777a776eca1cb4c923426949c997d8828`
- Balance: ~264,572 ADA

## Datum Comparison

### swap_a_to_b vs on-chain (db08967b)

**On-chain order datum** (from order submission db08967b):
```
Constr(0, [
  Bytes("563e7c47b132925519e288732063410b4ae536bdde116cb63d5e7d9c47cf89886889a96dd2344ee2360897337b64dfcb605499e79082a39c"),
  Constr(3, [189858])
])
```

**Generated datum:**
```
Constr(0, [
  Bytes("636d0d0118a8933ac167d4c448150bb325deaf7a4fdfb44adc7f2f5ae39b5f40aa85fbc121a625d777a776eca1cb4c923426949c997d8828"),
  Constr(3, [500000])
])
```

Result: **100% structural match**. Only `user_creds` and `min_receive_b` differ (different users/amounts).

### Order output comparison (swap_a_to_b)

| Field | On-chain (db08967b) | Generated (swap_a_to_b) |
|-------|---------------------|------------------------|
| Address | `712c0f418d...` (order script) | `712c0f418d...` (same) |
| ADA | 347,146,228 lovelace | 101,900,000 lovelace |
| Tokens | none | none |
| Datum | datum_hash | inline datum |

### swap_b_to_a vs on-chain (94eee7c4)

**On-chain order datum** (decoded from batcher execution bee950d8):
```
Constr(0, [
  Bytes("636d0d0118a8933ac167d4c448150bb325deaf7a4fdfb44adc7f2f5ae39b5f40aa85fbc121a625d777a776eca1cb4c923426949c997d8828"),
  Constr(4, [349217946])
])
```

**Generated datum:**
```
Constr(0, [
  Bytes("636d0d0118a8933ac167d4c448150bb325deaf7a4fdfb44adc7f2f5ae39b5f40aa85fbc121a625d777a776eca1cb4c923426949c997d8828"),
  Constr(4, [300000000])
])
```

Result: **100% structural match**. Only `min_receive` differs (different swap amounts).

### Order output comparison

| Field | On-chain (94eee7c4) | Generated (swap_b_to_a) |
|-------|---------------------|------------------------|
| Address | `712c0f418d...` (order script) | `712c0f418d...` (same) |
| ADA | 3,900,000 lovelace | 3,900,000 lovelace |
| Token policy | `279c909f...` (SNEK) | `279c909f...` (SNEK) |
| Token name | `534e454b` | `534e454b` |
| Token qty | 209,715 | 200,000 (test value) |
| Datum | datum_hash | inline datum |

### add_liquidity vs on-chain

**On-chain order datum** (from 675c02 add-liq execution):
```
Constr(0, [
  Bytes(56),
  Constr(0, [39686267])
])
```

**Generated:**
```
Constr(0, [
  Bytes(56),
  Constr(0, [100000])
])
```

Result: **100% structural match**

### remove_liquidity vs on-chain

**On-chain order datum** (from 8da8e2 remove-liq execution):
```
Constr(0, [
  Bytes(56),
  Constr(1, [Constr(0, [538352626, 323721])])
])
```

**Generated:**
```
Constr(0, [
  Bytes(56),
  Constr(1, [Constr(0, [100000000, 50000])])
])
```

Result: **100% structural match**

### cancel vs on-chain (34a41f7f, ADA/SNEK)

The only tx that executes a script. Verified field-by-field against the real cancel
tx `34a41f7f12bbb280a476d52f3802aabdd8575cfe96c277c3d7b866302a12d3b0`:

| Element | On-chain (34a41f7f) | Generated (cancel) |
|---------|---------------------|--------------------|
| Order input | `dbae1392…#0` @ order script (3.9M ADA + LP), datum = RemoveLiquidity order | `order_utxo` @ order script, `datum_is: OrderDatum` |
| Redeemer | `Constr(1, [])` (spend purpose) | `OrderRedeemer::Cancel {}` → `Struct{constructor:1, fields:[]}` |
| Script | PlutusV1 **inline witness** (2703-byte bytecode), **no** reference script | `cardano::plutus_witness { version: 1, script: order_script }` |
| Script hash | `2c0f418d…` (order address) | blake2b-224(`0x01` ‖ `order_script`) = `2c0f418d…` ✓ (verified) |
| Output 0 | order value back to user (3.9M ADA + 2,068,746 LP) | `amount: order_input` → user |
| Output 1 | change = funding input − fee (334,621,700 + SNEK) | `amount: source - fees` → user |
| Collateral | 5 ADA from the user, with return | `collateral { from: User }` |
| Validity | `invalid_before: 184163622`, `invalid_after: 184174422` (~3h window) | `validity { since_slot, until_slot }` |

Result: **structural match**. The order validator's Cancel branch needs the owner's
signature, satisfied by spending the user's own funding input (its payment cred =
the first 28 bytes of the order datum's `user_creds`). Both validity bounds are set
(wallet TTL window); the caller passes `since_slot`/`until_slot` from the live tip.

**Live-resolved (2026-06-23, `trix invoke --skip-submit --profile mainnet`):** cancel
resolves end-to-end (tx hash `c00b2182…`). Decoded CBOR matches the design exactly:
2 inputs (order `b05f582d…#0` + user funding), out0 = the order's value back to the
user (3_900_000 lovelace + 1_230_830 SNEK, byte-identical to the order UTxO), out1 =
change, `invalid_before = 184163622` (since_slot) / `ttl = 184174422` (until_slot),
`script_data_hash` present, 1 collateral, witness `plutus_v1_script` = the 2703-byte
`order_script` (hash `2c0f418d…` ✓), redeemer for spend input 0 = `d87a80` =
`Constr(1, [])` = Cancel.

**4 of 5 txs live-resolve** (`--skip-submit`):
- `cancel` (hash `c00b2182…`), `swap_a_to_b` (ADA-only, hash `359fdbd4…`).
- `swap_b_to_a` (hash `8b9e9b08…`) — `input*` combined **3** wallet UTxOs; order out =
  3_900_000 lovelace + 200_000 SNEK, datum `Constr(0,[user_creds, Constr(4,[300000000])])`.
  Confirms the `order_min_ada + process_fee` fold **live**: 2_000_000 + 1_900_000 = 3.9M.
- `add_liquidity` (hash `5260bf2e…`) — `input*` combined **2** UTxOs; order out =
  501_900_000 lovelace (500M + process_fee) + 1_000_000 SNEK, datum `Constr(0,[…,
  Constr(0,[100000])])`.
- `remove_liquidity` (hash `9b0a133c…`) — **datum confirmed value-equivalent to a REAL
  on-chain remove-liq order.** The test wallet holds no ADA/SNEK LP token, so it was
  resolved with the **real datum values** of the order that `34a41f7f` cancelled
  (`dbae1392#0`: `user_creds 83aa2e52…`, `min_out_ada 128732965`, `min_out_token 76833`)
  and SNEK as the LP placeholder (`input*` combined 3 UTxOs; the LP token lives in the
  output *value*, not the datum, so the datum is unaffected). Generated inline datum:
  `d87982…83aa2e52…d87a81d879821a07ac4f251a00012c21` = `Constr(0,[creds, Constr(1,
  [Constr(0,[128732965, 76833])])])` — the **same Plutus value** as the on-chain order
  (datum_hash `df44a16a` = the *indefinite*-array encoding; tx3 emits *definite* arrays,
  hash `0f841fd8`). Only diff = the definite-vs-indefinite CBOR artifact (value-preserving,
  affects every tx3 datum). Order ADA = 3_900_000 (order_min_ada + process_fee). **All 5
  txs now confirmed** (4 live-resolved end-to-end + remove_liquidity datum-matched).

**Script-bytes note:** tx3 hashes the `script:` value directly as `H(tag ‖ bytes)`,
so `order_script` must be the **doubly-wrapped** CBOR (the bytes Koios returns as
`script_info.bytes` / `tx_info.plutus_contracts[].bytecode`, starting `590a8c…` for
ADA/SNEK), **not** the unwrapped flat program. Confirmed: `H(0x01 ‖ full bytecode)`
= the order script hash, whereas stripping the `590a8c` header does not.

## Expected Differences

| Difference | Reason |
|------------|--------|
| Datum format (hash vs inline) | tx3 generates inline datums; on-chain VyFi uses datum hashes. Both are valid for the validator. |
| Input UTxOs | Different wallets = different UTxOs |
| Token quantities | Test values vs real swap amounts |
| Fees | `--skip-submit` uses placeholder fees |
| User change outputs | Different wallet UTxO structure |
| Witnesses | `--skip-submit` produces unsigned txs |

## Protocol Configuration

### Instance-level (shared across all pools)
- Process fee: 1,900,000 lovelace (1.9 ADA)
- Operator token: policy `4d07e0ceae00e6c53598cea00a53c54a94c6b6aa071482244cc0adb5`

### Per-pool (provided in invoke-args JSON)
- `orderscript`: the pool's order address
- Token policy, name (for swap_b_to_a, add_liquidity)
- LP policy, name (for remove_liquidity)
- `order_script`: the pool's order validator CBOR (cancel only — PlutusV1,
  doubly-wrapped, from Koios `script_info` / the VyFi API)

### Per-call (user-specific)
- `user`: user wallet address
- `user_creds`: payment_cred || stake_cred (56 bytes hex)
- Swap/deposit amounts, min receive values
- `order_min_ada` (swap_b_to_a, remove_liquidity): min-UTxO ADA buffer; the tx
  adds `process_fee` from env

### `order_min_ada` — why it is NOT inlined as `min_utxo(output) + process_fee`

The TX3-0.23 plan proposed inlining the token-order ADA as
`min_utxo(output) + process_fee`. **Rejected** (same trap as bodega `total_lovelace`
and aquarium `payment_token_qty`): across 21 real token-deposit orders (MIN, SNEK,
HOSKY, FREN, SICK pools) the ADA above `process_fee` is **mostly 2,000,000 but
varies** — 1,050,000 / 2,000,340 / 2,100,000 also seen. It is a frontend/wallet
choice, not the protocol's computed `min_utxo` (which for these outputs resolves to
~1.3M, diverging from the dominant 3.9M). So `order_min_ada` stays a caller param.
What *was* applied: fold the env `process_fee` into the order amount
(`Ada(order_min_ada + process_fee)`), making token orders symmetric with the
ADA-side orders (which already add `process_fee`) and dropping the need for the
caller to pre-sum the fee. Default `order_min_ada = 2_000_000` → 3.9M, the on-chain
mode.

## Pool Data Source

All 294 pools queryable at: `GET https://api.vyfi.io/lp?networkId=1`

Returns per pool:
- `poolValidatorUtxoAddress`: pool script address
- `orderValidatorUtxoAddress`: order script address (= `orderscript` party)
- `lpPolicyId-assetId`: LP token policy-name
- `json`: pool configuration (token pair info)
- `pair`: human-readable pair name (e.g., "ADA/SNEK")

## How to Reproduce

```bash
cd protocols_tx3/vyfi

# Swap ADA → Token
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/swap_a_to_b.json

# Swap Token → ADA
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/swap_b_to_a.json

# Add liquidity
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/add_liquidity.json

# Remove liquidity
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/remove_liquidity.json

# Cancel an order (the only script-executing tx; set since/until to the live tip,
# and order_utxo/order_script to an order you own + its pool's validator)
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/cancel.json
```

### Resolving token-deposit orders (`input*` + wallet holdings)

All order txs use `input* source` (it was `input source` initially): the resolver may
combine **multiple** wallet UTxOs to gather the deposit tokens + ADA. With a single
input the resolver needs one UTxO holding *both* the tokens and the ADA, which fails
on a fragmented wallet (`input not resolved: source`, `support_many: false`).

To **live-resolve** a token-deposit order the wallet must actually hold the asset:
- `swap_b_to_a` needs ≥ `token_amount` of the token (e.g. 200_000 SNEK),
- `add_liquidity` needs ≥ `token_amount` of the token (e.g. 1_000_000 SNEK),
- `remove_liquidity` needs ≥ `lp_amount` of that pool's LP token.

`swap_a_to_b` (ADA-only) and `cancel` (tokens come from the order input) resolve
without holding any pool token. If a token-deposit order still fails after `input*`,
the wallet doesn't hold enough of that asset — lower the amount in the invoke-args to
match the balance, or treat it as TIR-verified only.

> Headless verification (no TTY): `trix inspect tir --tx <name> --pretty --profile
> mainnet`. cancel's TIR pins redeemer `Constr(1,[])`, `plutus_witness` version 1 +
> `order_script` param, `validity {since,until}`, and outputs `order_input` / `source
> - fees`. swap_b_to_a & remove_liquidity differ from their pre-edit baseline ONLY by
> `order_ada` → `Add(order_min_ada, process_fee)`; swap_a_to_b & add_liquidity are
> byte-identical.
