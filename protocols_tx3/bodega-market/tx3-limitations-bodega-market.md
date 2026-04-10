# TX3 Limitations Found During Bodega Market Implementation

Discovered with trix 0.21.1 (2026-03-13).

---

## Active Limitations (impact on production use)

### 1. No Multiplication or Division Operators

The grammar only supports `+` and `-` for data expressions (`data_infix = _{ data_add | data_sub }`). Bodega Market requires arithmetic for fee and payment calculations:

```
payment = buy_amount * unit_price
admin_fee = buy_amount * admin_fee_percent * 1_000_000 / 10_000
total = payment + admin_fee + batcher_fee + envelope_amount
```

**Impact:** Forces `total_lovelace` to be pre-computed by the API caller. This leaks protocol internals (fee formula, AMM pricing) to the caller layer. Same applies to `unit_price` (LMSR price per share).

**Workaround applied:** Added `total_lovelace` and `unit_price` as tx parameters. The caller computes them off-chain.

**Affected txs:** `buy_position_yes`, `buy_position_no`, `sell_position_yes`, `sell_position_no`

### 2. No Dynamic-Length Input/Output Lists (Batch Patterns)

tx3 requires each input/output to be declared statically. There's no way to express "for each position in the batch, include an input and output".

**Impact: CRITICAL** — Makes all batcher/admin operations impossible to model. The Aiken redeemer carries `pos_indices: List<(Int, Int)>` — a dynamic list of (input_idx, output_idx) pairs. Each pair maps a consumed position UTxO to a user payout output.

| Tx not implemented | Aiken Redeemer | Why |
|----|---------------|-----|
| Process Buy (batch) | `PredictionRedeemer::Apply` | Dynamic list of position inputs/outputs |
| Process Reward (batch) | `PredictionRedeemer::Reward` | Dynamic list + burn + oracle reference |
| Process Refund (batch) | `PredictionRedeemer::Refund` | Dynamic list + burn |
| Withdraw Admin Fees | `PredictionRedeemer::WithdrawFee` | Multi-validator tx |
| Close Market | `PredictionRedeemer::Close` | Multi-validator consume + NFT transfer |

**Workaround:** None — these 5 transactions cannot be implemented in tx3.

### 3. No Tuple Types — `List<(ByteArray, Int)>` Can't Be Represented

tx3 has `List<T>` and `Map<K,V>` but no tuple type `(A, B)`. The on-chain `PredictionDatum` in the GitHub V2 source uses `predictions: List<(ByteArray, Int)>` for candidate/amount pairs.

**Workaround applied:** The on-chain V2 repo type was not used directly. The actual deployed contract uses individual fields (`yes_shares`, `no_shares`, `yes_price`, `no_price`) instead of a list of tuples, so this was modeled as a flat record `PredictionDatum` with 7 fields. No data loss.

**Impact:** Low for this protocol (deployed contract structure avoids the issue). Would block modeling the V2 GitHub source faithfully.

### 4. Custom Types (Enums) Cannot Be Passed as Parameters

`from_json()` in `tx3-resolver/src/interop.rs` only supports 5 types: Int, Bool, Bytes, Address, UtxoRef. The `CandidateIdx` enum (`Candidate0` / `Candidate1`) cannot be passed dynamically.

**Impact:** Every tx that takes a candidate index must be duplicated into `_yes` / `_no` variants with the enum hardcoded in each.

**Workaround applied:** Split 3 user-facing txs into 6 variants:
- `buy_position_yes` / `buy_position_no`
- `submit_reward_yes` / `submit_reward_no`
- `sell_position_yes` / `sell_position_no`

The only difference between variants is `CandidateIdx::Candidate0 {}` vs `CandidateIdx::Candidate1 {}`. If enums were passable, all 6 would collapse into 3.

### 5. No Conditional Logic / Branching

Bodega supports markets where the payment token is either ADA or a custom token (`ProjectInfoDatum.payment_policy_id`). The position UTxO value structure differs:
- **ADA market:** lovelace includes payment + fee + envelope + batcher_fee
- **Token market:** lovelace = envelope + batcher_fee, token amount = payment + fee

tx3 has no `if/else` or `when` to branch on `payment_policy_id`.

**Workaround:** Duplicate txs into `_ada` / `_token` variants (same approach as the `_yes`/`_no` split for CandidateIdx). Would affect `buy_position` and `sell_position`.

**Status:** Not implemented — all active mainnet markets use ADA as payment token. If a token-payment market appears, the `_token` variants need to be added.

---

## Non-Issues / Clarifications

### Reference Inputs Work for Data UTxOs

The `reference` block was initially thought to only support script references. This is **wrong** — it works for any UTxO (data or script). Used in `create_market` to add `protocol_settings_ref` as a reference input:

```tx3
reference settings {
    ref: protocol_settings_ref,
}
```

However, tx3 cannot **read datum values** from reference inputs (field access on reference blocks is not supported). The reference is added to the transaction but its datum contents are opaque to tx3.

### `collateral_return` / `total_collateral` Not Generated

Real on-chain txs include explicit `collateral_return` (field 16) and `total_collateral` (field 17). The TRP does not generate these. This is cosmetic — transactions still validate.

---

## Summary

| # | Type | Description | Workaround | Impact |
|---|------|-------------|------------|--------|
| 1 | Limitation | No `*` / `/` operators | Caller pre-computes `total_lovelace` | 2 extra params per buy/sell tx |
| 2 | Limitation | No dynamic input/output lists | None — batcher txs not implementable | 5 txs blocked (all batcher/admin) |
| 3 | Limitation | No tuple types | N/A (deployed contract avoids tuples) | Low |
| 4 | Limitation | Enums not passable as params | Duplicate txs into `_yes`/`_no` variants | 3 txs → 6 variants |
| 5 | Limitation | No conditional logic | Not implemented (no token markets active) | Would need `_ada`/`_token` variants for buy/sell |

**Transactions blocked by tx3 limitations:** 5 out of 11 total (all batcher/admin operations).
**Transactions requiring workaround duplication:** 3 → 6 (enum variant split).
**Pending if token-payment markets appear:** `buy_position` and `sell_position` would need `_ada`/`_token` variants.
