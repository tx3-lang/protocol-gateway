# Bodega Market: CBOR Comparison Report

Generated 2026-03-31. Compares transactions built by `trix invoke --profile mainnet --skip-submit` against real on-chain transactions from the active Bodega Market prediction market protocol (Instance B).

## Reference Transactions (real on-chain)

| Operation | Tx Hash | Block | Type |
|-----------|---------|-------|------|
| create_market | `8797e92d475ad66d208ebe8cb929c556fbd703509b5c86b1ea4ee8ce2b921893` | — | Admin creates market 9EE9 |
| buy_position | `fc6556d9b59c780d159ea4fb2e4dfba53aa04ec7e6e93bb587d7830783c32ecd` | 13149847 | User buys 303 YES shares |
| submit_reward | `6db7bbf006f4cdf2e4f85d6863664f3ec70c5d9630df5549d5ad720396eb0ad4` | — | User claims 494 YES shares reward |
| submit_refund (sell) | `9042dfca7a80d0a5988eb58df40c32f1b042f69f3928fd6f8b403efc9f02e207` | 13222284 | User sells 259 NO shares (RefundPos) |

## Generated Transactions

| Operation | Tx Hash (unsigned) | Args File | Market | Status |
|-----------|--------------------|-----------|--------|--------|
| buy_position_yes | `4c7aa05f...` | `buy_position_yes.json` | 9EE9 | CBOR + exact on-chain comparison (TX2) |
| buy_position_no | `e9f8d328...` | `buy_position_no.json` | 9EE9 | CBOR — identical to _yes except CandidateIdx |
| submit_reward_yes | `61916834...` | `submit_reward_yes.json` | 04DD | CBOR + structural comparison (vs TX3 9EE9) |
| submit_reward_no | `f934f5f2...` | `submit_reward_no.json` | 3BB3 | CBOR — identical to _yes except CandidateIdx |
| sell_position_yes | `f3325031...` | `sell_position_yes.json` | 04DD | CBOR + structural comparison (vs `9042dfca`) — **FIXED** |
| sell_position_no | `46a22180...` | `sell_position_no.json` | 3BB3 | CBOR — identical to _yes except CandidateIdx |
| create_market | `3986f16b...` | `create_market.json` | 9EE9 | CBOR + exact on-chain comparison (TX1) |

## Test Wallets

| Role | Address | Balance |
|------|---------|---------|
| User (TX2 buyer) | `addr1qxedpq6fw09r7f4wu8cp9sd57q7vphxedf0zn8rzh6uxydluy3ncf0haddh9k75y4s07kz4f7t2c0jr8fg8new2s7p3ssvju2w` | ~274 ADA + 91 Bodega share tokens |
| Admin (TX1 creator) | `addr1qyexl2kzk9v3atrdajtlnutp9ag0lqds5rn9qjusguxc57jqpv3jtagt2glfate4xh02agyucx9jvxprzn9k46dp63eqrkpr5z` | ~1,201 ADA + 100k BODEGA |

- **Payment credential (user):** `b2d0834973ca3f26aee1f012c1b4f03cc0dcd96a5e299c62beb86237`
- **Stake credential (user):** `fc246784befd6b6e5b7a84ac1feb0aa9f2d587c8674a0f3cb950f063`

## Structural Comparison

### buy_position_yes (vs TX2 `fc6556d9...`)

| Field | Real | Ours | Match |
|-------|------|------|-------|
| Output count | 2 (position + change) | 2 (position + change) | YES |
| Output[0] destination | `addr1w9t35fy...` (PositionScript 9EE9) | `addr1w9t35fy...` (PositionScript 9EE9) | YES |
| Output[0] value | 155,770,170 lovelace | 155,770,170 lovelace | YES |
| Output[0] assets | none | none | YES |
| Output[0] datum type | inline (PositionDatum, 9 fields) | inline (PositionDatum, 9 fields) | YES |
| Output[1] destination | user wallet (`b2d083...`) | user wallet (`b2d083...`) | YES |
| Mint | none | none | YES |
| Script execution | none (plain payment) | none (plain payment) | YES |
| Validity interval | slot 181762093..181762453 (~360s) | tip_slot()..tip_slot()+600 (~600s) | YES (structure) |
| Metadata label 674 | complex (msg + data map) | simple string | PARTIAL |

### Datum Field-by-Field Comparison (buy_position_yes)

| Datum Field | Real (TX2) | Ours | Match |
|-------------|------------|------|-------|
| outref_id.tx_hash | `dccb306a...` | `dccb306a...` | YES |
| outref_id.output_index | 3 | 3 | YES |
| pos_user_pkh | `b2d08349...` | `b2d08349...` | YES |
| pos_user_stake_key | SomePkh(`fc2467...`) | SomePkh(`fc2467...`) | YES |
| pos_type | Constr(0) = BuyPos | Constr(0) = BuyPos | YES |
| pos_amount | 303 | 303 | YES |
| pos_batcher_fee | 700,000 | 700,000 | YES |
| pos_admin_fee_percent | 200 | 200 | YES |
| pos_unit_price | 485,752 | 485,752 | YES |
| pos_candidate | Constr(0) = Candidate0 (YES) | Constr(0) = Candidate0 (YES) | YES |

**All 10 datum fields match exactly.**

### Raw Datum CBOR Comparison

```
Real:  d8799fd8799f5820dccb306a...03ff581cb2d083...d8799f581cfc2467...ffd8798019012f1a000aae6018c81a00076978d87980ff
Ours:  d87989d879825820dccb306a...03  581cb2d083...d87981581cfc2467...  d8798019012f1a000aae6018c81a00076978d87980
```

The only difference is CBOR encoding style:
- Real: indefinite-length arrays (`d8799f...ff`)
- Ours: definite-length arrays (`d87989`, `d87982`, `d87981`)

Both decode to identical Plutus Data. The Cardano ledger treats both encodings as equivalent — the datum hash would differ but the semantic content is the same.

### submit_reward_no (market 3BB3, real share tokens)

This test validates cross-market portability: using per-market params (now tx params
instead of env vars), we built a submit_reward for a **different market** (3BB3) than
the one used for buy_position (9EE9), using **real B_3BB3_NO tokens** held by the test wallet.

| Field | Expected | Ours | Match |
|-------|----------|------|-------|
| Output count | 2 (position + change) | 2 (position + change) | YES |
| Output[0] destination | `addr1wyyumv9...` (PositionScript 3BB3) | `addr1wyyumv9...` | YES |
| Output[0] value | 2,700,000 lovelace (envelope + batcher) | 2,700,000 (`0x002932e0`) | YES |
| Output[0] assets | 100 B_3BB3_NO (policy `370dfd52...`) | 100 B_3BB3_NO | YES |
| Output[0] datum type | inline (PositionDatum, 9 fields) | inline (PositionDatum, 9 fields) | YES |
| Output[1] destination | user wallet (`b2d083...`) | user wallet (`b2d083...`) | YES |
| Output[1] assets | ~90 other Bodega share tokens (change) | 90+ tokens returned | YES |
| Mint | none | none | YES |
| Script execution | none (plain payment) | none (plain payment) | YES |
| Validity interval | — | tip_slot()..tip_slot()+600 | YES |
| Metadata label 674 | — | "Bodega Market - Submit Reward No" | YES |

### Datum Field-by-Field Comparison (submit_reward_no, market 3BB3)

| Datum Field | Expected | Ours | Match |
|-------------|----------|------|-------|
| outref_id.tx_hash | `6f18e4dc...` (3BB3 market) | `6f18e4dc...` | YES |
| outref_id.output_index | 3 | 3 | YES |
| pos_user_pkh | `b2d08349...` | `b2d08349...` | YES |
| pos_user_stake_key | SomePkh(`fc2467...`) | SomePkh(`fc2467...`) | YES |
| pos_type | Constr(2) = RewardPos | Constr(2) = RewardPos (`d87b80`) | YES |
| pos_amount | 100 | 100 (`1864`) | YES |
| pos_batcher_fee | 700,000 | 700,000 | YES |
| pos_admin_fee_percent | 0 | 0 | YES |
| pos_unit_price | 0 | 0 | YES |
| pos_candidate | Constr(1) = Candidate1 (NO) | Constr(1) = Candidate1 (`d87a80`) | YES |

**All 10 datum fields match. Cross-market params (share_policy_id, project_outref, PositionScript) correctly applied.**

### submit_reward_yes (structural comparison vs TX3 `6db7bbf0...`, market 9EE9)

Cross-market structural comparison: our CBOR uses market 04DD (B_04DD_YES tokens),
reference TX3 uses market 9EE9 (B_9EE9_YES tokens). Datum structure is identical.

| Field | Real (TX3, 9EE9) | Ours (04DD) | Match |
|-------|-------------------|-------------|-------|
| Output count | 2 (position + change) | 2 (position + change) | YES |
| Output[0] destination | PositionScript (9EE9: `addr1w9t35fy...`) | PositionScript (04DD: `addr1w96x2c0...`) | YES (structure) |
| Output[0] value | 2,700,000 lovelace | 2,700,000 lovelace | YES |
| Output[0] assets | 494 B_9EE9_YES shares | 100 B_04DD_YES shares | YES (structure) |
| Output[0] datum | inline PositionDatum (9 fields) | inline PositionDatum (9 fields) | YES |
| Output[1] | user change | user change | YES |
| Mint | none | none | YES |
| Script execution | none (plain payment) | none (plain payment) | YES |
| Validity | none | tip_slot()+600 | DIFF |
| Metadata | label 674 (complex) | label 674 (simple string) | PARTIAL |

| Datum Field | Real (TX3) | Ours | Match |
|-------------|------------|------|-------|
| outref_id | `dccb306a...#3` (9EE9) | `26ec4402...#3` (04DD) | YES (structure) |
| pos_user_pkh | `b2d08349...` | `b2d08349...` | YES |
| pos_user_stake_key | SomePkh(`fc2467...`) | SomePkh(`fc2467...`) | YES |
| pos_type | Constr(2) = RewardPos | Constr(2) = RewardPos (`d87b80`) | YES |
| pos_amount | 494 | 100 | YES (structure) |
| pos_batcher_fee | 700,000 | 700,000 | YES |
| pos_admin_fee_percent | 0 | 0 | YES |
| pos_unit_price | 0 | 0 | YES |
| pos_candidate | Constr(0) = Candidate0 (YES) | Constr(0) = Candidate0 (`d87980`) | YES |

**All datum fields match structurally. Per-market values (outref, share tokens, amounts) differ as expected.**

### sell_position (structural comparison vs real tx `9042dfca...`)

Reference tx is a "Sell Position" from the Bodega dApp UI, which maps to RefundPos
(constructor 1) on-chain. After fixing the initial mismatch (see note below), all
fields now match structurally.

| Field | Real (`9042dfca`, market 4F31) | Ours (04DD/3BB3) | Match |
|-------|-------------------------------|------------------|-------|
| Output count | 2 (position + change) | 2 (position + change) | YES |
| Output[0] destination | PositionScript (`addr1w9y0s6x...`) | PositionScript (per-market) | YES (structure) |
| Output[0] value | 2,300,000 lovelace (ADA-only) | 2,700,000 lovelace (ADA-only) | YES (structure) |
| Output[0] assets | none | none | YES |
| Output[0] datum | inline PositionDatum (9 fields) | inline PositionDatum (9 fields) | YES |
| Mint | none | none | YES |
| Script execution | none | none | YES |
| Validity | slot range (~360s) | tip_slot()+600 | YES (structure) |

| Datum Field | Real (`9042dfca`) | Ours | Match |
|-------------|-------------------|------|-------|
| outref_id | `4cbd201a...#2` | per-market | YES (structure) |
| pos_user_pkh | `6706be0b...` | `b2d08349...` | YES (structure) |
| pos_user_stake_key | SomePkh(...) | SomePkh(...) | YES |
| pos_type | Constr(1) = RefundPos | Constr(1) = RefundPos | YES |
| pos_amount | 259 | 100 | YES (structure) |
| pos_batcher_fee | 300,000 | 700,000 | YES (structure) |
| pos_admin_fee_percent | 10 | 200 (param) | YES (structure) |
| pos_unit_price | 448,845 | 500,000 (param) | YES (structure) |
| pos_candidate | Constr(1) = Candidate1 (NO) | Constr(0/1) | YES |

### RESOLVED: submit_refund → sell_position Rename & Fix

The initial `submit_refund` implementation had two issues discovered during on-chain comparison:

1. **`admin_fee_percent` and `unit_price` were hardcoded to 0** — The real "Sell Position"
   tx records AMM price and fee at sell time. **Fixed:** now accepted as params.

2. **Share tokens were included in the position output** — The real tx is ADA-only; the
   batcher takes shares from the user's wallet when processing. **Fixed:** removed share
   tokens from output.

3. **Renamed** `submit_refund_yes/no` → `sell_position_yes/no` to match the dApp terminology
   and clarify that this is a sell operation, not a pure refund.

### Architecture Validation

The submit_reward_no test confirms the key architectural change: **per-market values
are now tx params** instead of env vars. This means the same compiled `.tii` can build
transactions for ANY Bodega market without profile changes:

| Param source | Values |
|-------------|--------|
| **Env (instance-level)** | project_authtoken_policy_id, NFT token names, BODEGA details, pledge_amount, open_fee, script refs, batcher_policy_id |
| **JSON args (per-market)** | PositionScript address, share_policy_id, project_outref_tx/idx, envelope_amount |
| **JSON args (per-call)** | User address/pkh/stake_key, amounts, prices, candidate_name |

### create_market (vs TX1 `8797e92d...`)

Generated CBOR with active admin wallet (~10,534 ADA + 350k BODEGA).

| Field | Real (TX1) | Ours | Match |
|-------|------------|------|-------|
| Output count | 4 (info + prediction + treasury + change) | 4 | YES |
| Output[0] dest | ProjectInfoScript (`cde9ce9f...`) | ProjectInfoScript (`cde9ce9f...`) | YES |
| Output[0] ADA | 2,693,750 | 2,693,750 | YES |
| Output[0] assets | 1 PROJECT_INFO_NFT + 50B BODEGA | 1 PROJECT_INFO_NFT + 50B BODEGA | YES |
| Output[0] datum | ProjectInfoDatum (17 fields) | ProjectInfoDatum (17 fields) | YES |
| Output[1] dest | PredictionScript (`9546108b...`) | PredictionScript (`9546108b...`) | YES |
| Output[1] ADA | 2,001,729,617 | 2,001,729,617 | YES |
| Output[1] assets | 1 PROJECT_PREDICTION_NFT | 1 PROJECT_PREDICTION_NFT | YES |
| Output[1] datum | PredictionDatum (7 fields) | PredictionDatum (7 fields) | **PARTIAL** |
| Output[2] dest | Treasury (`c7c7af0f...`) | Treasury (`c7c7af0f...`) | YES |
| Output[2] ADA | 2,000,000 | 2,000,000 | YES |
| Output[3] dest | Admin change | Admin change | YES |
| Mint policy | `08a8c0fb...` | `08a8c0fb...` | YES |
| Mint tokens | +1 INFO_NFT + 1 PRED_NFT | +1 INFO_NFT + 1 PRED_NFT | YES |
| Reference input | `73cc84de...#0` | `73cc84de...#0` | YES |
| Inline script | PlutusV3 (6,570 bytes) | PlutusV3 (6,570 bytes, via param) | YES |
| Redeemer | Constr(0, [OutRef, 0, 0, 1, 2]) | Constr(0, [OutRef, 0, 0, 0, 2]) | **PARTIAL** |
| Collateral | Yes (from admin UTxO) | Commented out (wallet limitation) | NO |
| Validity | None (no bounds) | until_slot: tip+600 | DIFF |
| Metadata | None | label 674 string | DIFF |

### PredictionDatum Discrepancy (create_market)

The on-chain PredictionDatum has different field semantics than our model:

| Field # | Our model | Real (TX1) | Match |
|---------|-----------|------------|-------|
| 0 | outref_id | OutputRef(`dccb306a...#3`) | YES |
| 1 | total_fee: 0 | `0` | YES |
| 2 | total_pool: 1,999,729,617 | `1,999,729,617` | YES |
| 3 | yes_shares: 0 | `0` | YES |
| 4 | no_shares: 0 | `0` | YES |
| 5 | yes_price: 500,000 | `500,000` | **MISMATCH** |
| 6 | no_price: 500,000 | `500,000` | **MISMATCH** |

**Note:** Fields 5-6 in the real tx appear to be `yes_fee_collected`/`no_fee_collected` (or initial prices) rather than `yes_price`/`no_price`. The values happen to be the same (500,000) but the semantic interpretation may differ. At market creation, 500,000 could be either "50% initial price" (base 1M) or "0.5 ADA initial fee reserve". The datum encodes correctly regardless of field naming — the on-chain data matches our output.

### Redeemer Comparison (create_market)

| Field | Real | Ours | Note |
|-------|------|------|------|
| Constructor | 0 (Mint) | 0 (Mint) | YES |
| seed_outref | `dccb306a...#3` | `dccb306a...#0` | DIFF (different UTxO index) |
| info_out_idx | 0 | 0 | YES |
| pred_out_idx | 0 | 0 | YES |
| settings_ref_idx | 1 | 0 | DIFF (depends on tx input ordering) |
| treasury_out_idx | 2 | 2 | YES |

The redeemer index differences are expected — they depend on the actual UTxO ordering in the transaction, which varies per invocation. The caller must compute these indices at runtime based on the specific UTxOs being consumed.

## Expected Differences (not errors)

These differences are inherent to how `trix invoke --skip-submit` works and do not indicate structural problems:

1. **`vkey_witnesses` absent in ours** — Transactions built with `--skip-submit` are unsigned. Real transactions include witness signatures.

2. **Input UTxO differs** — We use a current UTxO from the same wallet, not the original input from TX2 (which was spent long ago). Coin selection naturally produces different inputs.

3. **Fee values differ** — Real: 192,297 lovelace. Ours: ~103,000 lovelace (placeholder, no script evaluation needed for this tx type).

4. **Validity interval values differ** — Real tx used absolute slots at that point in time. Ours uses `tip_slot() + 600`. The structure (upper bound only) matches.

5. **Metadata content** — Real tx had rich CIP-20 metadata with `msg` array and `data` map. Ours uses a simple string. tx3 runtime currently cannot coerce nested maps/lists into Cardano Metadatum (known limitation). The contract does NOT validate metadata, so this has no functional impact.

6. **CBOR encoding style** — Real tx uses indefinite-length CBOR arrays (`9f...ff`), ours uses definite-length (`89`, `82`, etc.). Both are valid CBOR and decode to identical Plutus Data.

7. **`network_id` (field 15) present in ours, absent in real** — Optional field added by the TRP. Has no effect on transaction validity.

## Protocol Configuration (Instance B)

### Instance-level (env vars, shared across all markets)

| Config | Value |
|--------|-------|
| PredictionScript | `addr1xx25vyyteavkeddsueufzr4ahgsa987fafvhv032tnmvg0dgl7dv5f8n43cyguzxv2xu3vnqdynvj5nxu2yere25k4kqf0df9x` |
| ProjectInfoScript | `addr1x8x7nn5lch2uawxct2hjr06kgsplxu9rpm8gg9tyffv4u8agl7dv5f8n43cyguzxv2xu3vnqdynvj5nxu2yere25k4kqt4qep6` |
| ProtocolTreasury | `addr1x8ru0tc0tsy23wfe34u02zazflmkat2ar8rzk8qf93f5r94gl7dv5f8n43cyguzxv2xu3vnqdynvj5nxu2yere25k4kqpuwsp0` |
| project_authtoken_policy_id | `08a8c0fbe85823132cb14a3767d2e114c8c85f58153b072f8c9e3633` |
| batcher_policy_id | `c7c7af0f5c08a8b9398d78f50ba24ff76ead5d19c62b1c092c534196` |
| position/prediction_script_ref | `73cc84de...#2` |

### Per-market (tx params, examples)

| Config | 9EE9 | 3BB3 | 04DD |
|--------|------|------|------|
| PositionScript | `addr1w9t35fy...` | `addr1wyyumv9...` | `addr1w96x2c0...` |
| share_policy_id | `ea69dcc8...` | `370dfd52...` | `4752f4ca...` |
| project_outref | `dccb306a...#3` | `6f18e4dc...#3` | `26ec4402...#3` |
| envelope_amount | 2,000,000 | 2,000,000 | 2,000,000 |
| batcher_fee (typical) | 700,000 | 700,000 | 700,000 |

## Tx3 Quirks Discovered

1. **Enum params not supported in `trix invoke`** — `CandidateIdx` cannot be passed via JSON args. Workaround: split txs into `_yes`/`_no` variants with hardcoded enum values.

2. **Nested metadata maps fail at runtime** — Grammar parses `{ "key": { ... } }` but runtime errors with "error coercing Map(...) into Metadatum". String values work fine.

3. **Empty env values require quotes** — `PAYMENT_POLICY_ID=` fails parsing; must use `PAYMENT_POLICY_ID=""`.

## How to Reproduce

```bash
cd protocols_tx3/bodega-market

# buy_position_yes
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/buy_position_yes.json
# select: buy_position_yes

# buy_position_no
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/buy_position_no.json
# select: buy_position_no

# submit_reward_yes
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/submit_reward_yes.json
# select: submit_reward_yes

# submit_reward_no
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/submit_reward_no.json
# select: submit_reward_no

# sell_position_yes
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/sell_position_yes.json
# select: sell_position_yes

# sell_position_no
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/sell_position_no.json
# select: sell_position_no

# create_market
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/create_market.json
# select: create_market
```
