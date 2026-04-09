# VyFi DEX — Comparison Report

## Reference Transactions (on-chain)

| Type | TX Hash | Pool |
|------|---------|------|
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

## Test Wallet

- Address: `addr1q93k6rgprz5fxwkpvl2vgjq4pwejth400f8aldz2m3lj7khrnd05p259l0qjrf396am6wahv5895ey35y62fexta3q5q3cc3k8`
- Payment cred: `636d0d0118a8933ac167d4c448150bb325deaf7a4fdfb44adc7f2f5a`
- Stake cred: `e39b5f40aa85fbc121a625d777a776eca1cb4c923426949c997d8828`
- Balance: ~264,572 ADA

## Datum Comparison

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

### Per-call (user-specific)
- `user`: user wallet address
- `user_creds`: payment_cred || stake_cred (56 bytes hex)
- Swap/deposit amounts, min receive values

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
```
