# VyFi DEX Protocol Research

## Overview

VyFi is an AMM DEX on Cardano with a two-step order model:
1. **User submits order** → sends funds + datum to an order script address (no scripts executed)
2. **Batcher processes order** → consumes order + pool UTxO, updates pool, distributes results (scripts executed)

Users only build the order-submission transactions. Batcher processing is handled by VyFi infrastructure.

## Protocol Architecture

### VYFI Token
- Policy ID: `804f5544c1962a40546827cab750a88404dc7108c0f588b72964754f`
- Asset Name: `56594649` (VYFI)
- Decimals: 6

### Per-Pool Components

Each pool has its own unique:
- **Pool Address** (`addr1z...`): holds liquidity (ADA + Token + Pool NFT)
- **Order Address** (`addr1w...`): holds pending user orders
- **Pool Validator Script**: validates pool state transitions
- **Order Validator Script**: validates order processing/cancellation
- **LP Minting Policy**: mints/burns LP tokens
- **Pool NFT**: unique NFT identifying the pool (policy varies per pool)

### Shared Components
- **Operator Token**: policy `4d07e0ceae00e6c53598cea00a53c54a94c6b6aa071482244cc0adb5`, name `VyFi_Credential`
- **Fee Settings**: barFee=10, processFee=1,900,000 lovelace, liqFee=20

### Example Pools

| Pool | Pool Address | Order Address | LP Policy |
|------|-------------|---------------|-----------|
| ADA/SNEK | `addr1zy3jcyykdnjd3enu96hp0w6hct85s499w5y6w5hmk0qzh50qy02w7ayk0lyyrf080l5zusdpkg8se9x7fcke6vaz42lqwjjflq` | `addr1wykq7svdj3ys97tyyn9uv55r5x9azmef8p4yjy4a50v0r7qmspxts` | `630e29291e9701aa47fbc22892333ed70975fe9f03d23d92ddb1c0f0` |
| ADA/USDA | `addr1z955fyznplf6hnuf4tgzwkpjqwe8x5yq7n5yrehp3xkypm9ksyd8pn7amnr48geat0yft0uezfunealzy4ghl0cayp4snfzxkj` | `addr1w9wghyyx2kh4uekkvdj2jdfztggfenvt4nh9wnmn5s2gd7ga6tpvc` | `e4b858e5531da1ce3613750503ab1367fb2982320f107489c98402f6` |
| ADA/MIN | `addr1z85nas88ngr7dvlnpjxf78n0gclgvd3cfr66jmtx5hp9h2x08470jem2z3k5yap4fddtxvykxxv2tmr83x06vklv5vysntgj0h` | `addr1wyv43n3dwy0emfysk4u7c4j6n5n3yl2xfeys3jpzq255ccsl7yskf` | `a72dc0efe67f5bba75c782a30ee2f77cf3016b18c1af47942c321f67` |
| ADA/HOSKY | `addr1z84549cnjrpalhjsprnjynx3m0hkrgt6j23wfu2wl27fcww60wckas4haxwwclas0g39cc8cvt2r8yalrfa9e8vxx92qrznltm` | `addr1wx8fk923cyzhlptl2th8x0uqwml8j8t3q2e99t0vfn4ewjszp8ksm` | `f5f91e62e38fb5689e4e03e19c41a773a079684f2dd4b77be24caf5d` |

### Pool Configuration (from API `/lp?networkId=1` json field)

```json
{
  "bAsset": {"tokenName": "SNEK_hex", "currencySymbol": "snek_policy"},
  "mainNFT": {"tokenName": "", "currencySymbol": "pool_nft_policy"},
  "operatorToken": {"tokenName": "VyFi_Credential", "currencySymbol": "4d07e0ceae00e6c53598cea00a53c54a94c6b6aa071482244cc0adb5"},
  "lpTokenName": {"unTokenName": "VyFi_ADA/SNEK_LP"},
  "aAsset": {"tokenName": "", "currencySymbol": ""},  // lovelace
  "feesSettings": {"barFee": 10, "processFee": 1900000, "liqFee": 20},
  "stakeKey": "hex_stake_key"
}
```

## On-Chain Datum Structures

### Pool Datum (constructor 0)
```
Constr(0, [
  Int,    // field 0: unclear meaning (reserves tracking?)
  Int,    // field 1: unclear (LP supply related?)
  Int,    // field 2: unclear (ADA reserves?)
])
```

Example (ADA/SNEK): `[3150571, 1200, 1323523146]`

### Order Datum (constructor 0)
```
Constr(0, [
  Bytes(56),    // user address = payment_cred (28 bytes) + stake_cred (28 bytes)
  OrderType,    // varies by constructor
])
```

#### Order Types (field 1)

**Constructor 0 - Add Liquidity:**
```
Constr(0, [Int])  // desired/min LP tokens
```
Example: `Constr(0, [39686267])` = min 39,686,267 LP tokens

**Constructor 1 - Remove Liquidity:**
```
Constr(1, [
  Constr(0, [Int, Int])  // [min_token_a (ADA), min_token_b]
])
```
Example: `Constr(1, [Constr(0, [538352626, 323721])])` = min 538 ADA, min 323721 SNEK

**Constructor 3 - Swap A to B (ADA → Token):**
```
Constr(3, [Int])  // min receive token_b
```
Example: `Constr(3, [34393107])` = min 34,393,107 units of token B

**Constructor 4 - Swap B to A (Token → ADA):**
```
Constr(4, [Int])  // min receive ADA (lovelace)
```
Example: `Constr(4, [349217946])` = min 349,217,946 lovelace

**Constructor 2 - unknown (possibly zap or single-sided add)**

### Order Redeemers

- **Constructor 0**: Process/Execute (used by batcher)
- **Constructor 1**: Cancel (used by user)

## Transaction Types

### 1. Swap Order Submission (User TX - no scripts)

**ADA → Token swap:**
- Output to order address: `processFee (1,900,000) + swap_amount` lovelace + datum_hash
- Datum: `Constr(0, [user_creds, Constr(3, [min_receive])])`

**Token → ADA swap:**
- Output to order address: `processFee (1,900,000) + min_ada` lovelace + tokens + datum_hash
- Datum: `Constr(0, [user_creds, Constr(4, [min_receive_ada])])`

Reference txs:
- Order submission: `94eee7c4678c2ef90e65ad13cac0ffee6f6a5507397ffb7c31eecc7bdae1707b`
- Swap execution: `bee950d8767ffae7b0ee6d92279e447cad5c7f569f3715cac4cc196d6e39d7b3`

### 2. Add Liquidity Order (User TX - no scripts)

- Output to order address: `processFee + ada_amount` lovelace + tokens + datum_hash
- Datum: `Constr(0, [user_creds, Constr(0, [min_lp_tokens])])`

Reference txs:
- Add liq execution: `675c02603d413248e93727dd5516ebedede538fa5195b3bab57e9561bc2e4af9`

### 3. Remove Liquidity Order (User TX - no scripts)

- Output to order address: `processFee + min_ada` lovelace + LP tokens + datum_hash
- Datum: `Constr(0, [user_creds, Constr(1, [Constr(0, [min_ada, min_token_b])])])`

Reference txs:
- Remove liq execution: `8da8e2e4dc74432573b857f2a035fc21ea3404b4a459f82191defd8c1799095c`

### 4. Cancel Order (User TX - script execution)

- Consumes order UTxO from order address
- Redeemer: `Constr(1, [])` (cancel)
- Has `invalid_before` validity constraint (time-based cancellation)
- Returns funds to user

Reference txs:
- Cancel: `34a41f7f12bbb280a476d52f3802aabdd8575cfe96c277c3d7b866302a12d3b0`

## Pool Data Source

All 294 pools available at: `GET https://api.vyfi.io/lp?networkId=1` (no auth required)

Each pool returns:
- `unitsPair`: token pair identifier
- `poolValidatorUtxoAddress`: pool script address
- `orderValidatorUtxoAddress`: order script address
- `lpPolicyId-assetId`: LP token policy + name
- `json`: pool configuration datum (bAsset, mainNFT, etc.)
- `pair`: human-readable pair name
- `isLive`: whether pool is active

## Script Hashes (ADA/SNEK pool)

- Order validator: `2c0f418d944902f96424cbc65283a18bd16f29386a4912bda3d8f1f8`
- Pool validator: `232c10966ce4d8e67c2eae17bb57c2cf4854a57509a752fbb3c02bd1`
- LP minting policy: `630e29291e9701aa47fbc22892333ed70975fe9f03d23d92ddb1c0f0`

## Key Observations

1. **No reference scripts**: Scripts are embedded as tx witnesses, not reference scripts
2. **Per-pool addresses**: Each pool has unique pool + order addresses (294 pools = 588 unique addresses)
3. **Two-step model**: Users only submit orders; batcher processes them against the pool
4. **Order ADA**: Process fee is 1,900,000 lovelace (1.9 ADA)
5. **Metadata**: Batcher txs include metadata key 674: `"msg": "VyFi: LP Order Process"` or `"VyFi: Swap Order Process"`
6. **Collateral**: Only required for batcher (processing) and cancel txs, NOT for order submissions
