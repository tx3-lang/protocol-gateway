# TX3 Limitations Found During Bodega Market Implementation

Discovered with trix 0.21.1 (2026-03-13). Updated 2026-04-14 with tx3c v0.17.0 / trix 0.22.0 findings.

---

## Active Limitations (impact on production use)

### 1. Position total needs the off-chain LMSR cost — NOT inlinable with `*`/`/`

> **Update 2026-06-22 (tx3 0.23 / trix 0.26):** `*` and `/` now exist (#339/#340, 0.22). We
> investigated inlining `total_lovelace`, reverse-engineered the **real** deployed formula
> against on-chain txs + the local cost analysis, and decided to **keep `total_lovelace` as a
> caller param.** Reasoning below.

**The real deployed formula** (the mainnet contract is LMSR-based and is **not** the
open-source GitHub v2 — they differ; see `bodega-market-costs.md` §1, verified vs 5 trades):

```
total = LMSR_cost
      + floor( LMSR_cost * (pi_admin_fee_percent + pos_admin_fee_percent) / 10_000 )
      + batcher_fee + envelope_amount
```

Verified **exact** on tx `f57e74aa…` (market 5637_SPCX, NO):
`147 149 730 + floor(147 149 730 × 400 / 10 000 = 5 885 989) + 700 000 + 2 000 000 =
155 735 719` ✓. (Both admin percents were 200 → combined 4 %, matching the docs' "~4 % of
volume". The earlier wrong inline used `buy_amount × admin% × 1e6/1e4` = 6 780 000 instead of
5 885 989 → that was the +894 011 error.)

**Two walls to inlining it:**

1. **`LMSR_cost ≠ buy_amount × unit_price`.** The stored `unit_price` is a rounded,
   informational average. The amount that actually enters the pool is the exact LMSR cost
   `C(q+Δ) − C(q)`, `C = b · ln Σ exp(qᵢ/b)`. They diverge — e.g. trade *197 NO*:
   `amount × unit_price` = 97 245 307 but real LMSR cost = 97 142 992 (off by **102 315**).
   So the cost **must** come from the off-chain ln/exp engine; it can't be reconstructed in-tx.

2. **Even passing `LMSR_cost` as a param and inlining the rest is not worth it:**
   - No compute saving — the caller runs ln/exp once regardless and already has the total.
   - No fewer params — the admin fee needs `pi_admin_fee_percent` (ProjectInfoDatum), which
     **can't be read from the reference datum in an amount expression** (limitation #6) → it
     becomes an extra param. `total_lovelace` (1 param) → `lmsr_cost` + `pi_admin%` (≥2).
   - Risk — the combined-rate single-floor formula is verified on one config (both = 200);
     BODEGA-holder discounts (`pos_admin% = 10`) are unverified, and a 1-lovelace error makes
     the position **unprocessable** by the batcher. The deployed contract is closed-source, so
     the formula can't be confirmed from source — only reverse-engineered.

**Workaround (kept):** `total_lovelace` **and** `unit_price` remain tx parameters, both
computed off-chain by the caller's LMSR engine.

**Affected txs:** `buy_position` (and the per-trade price in `sell_position`).

**Lesson:** `*`/`/` only help when the polynomial is the *actual, verifiable* on-chain math.
Here it isn't (closed-source LMSR contract; the cost itself needs ln/exp). The old "verified
on-chain" comment was circular (back-computed from the same formula). Always validate against
a **real** tx — that is what caught both the wrong admin formula and the cost approximation.

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

### 6. Reference Datum Fields Only Usable in Datum Construction

Discovered with tx3c v0.17.0 / trix 0.22.0 (2026-04-14).

tx3c v0.17.0 (#318) added `datum_is` on reference blocks, allowing typed field access on reference input datums. However, the **resolver** only supports these fields inside **output datum construction** — not in **amount expressions** (`Ada(...)`, `AnyAsset(...)`, `min_amount`, change calculations).

```tx3
reference project_info {
    ref: project_info_ref,
    datum_is: ProjectInfoDatum,
}

// WORKS — datum field in output datum
output {
    datum: PositionDatum {
        outref_id: project_info.outref_id,  // OK
    },
}

// FAILS — datum field in amount expression
output {
    amount: Ada(project_info.pi_envelope_amount + batcher_fee_amount),  // resolver error
}

// FAILS — datum field in AnyAsset
locals {
    shares: AnyAsset(project_info.pi_share_policy_id, project_info.candidate_yes_name, amount),  // resolver error
}
```

**Error:** `expected assets, got EvalBuiltIn(Add(Assets([...]), EvalBuiltIn(Property(EvalCoerce(IntoAssets(EvalParam(ExpectInput(...))))), Number(13)))))`

**Impact:** Fields that would eliminate caller params (`pi_envelope_amount`, `pi_share_policy_id`, `candidate_yes/no_name`) cannot be read from reference datums because they're used in amount calculations. Only `outref_id` (used exclusively in datum construction) can be read from the reference.

**Workaround applied:** Keep `envelope_amount`, `share_policy_id`, and `candidate_name` as caller-provided tx params. Use `datum_is` only for `outref_id` in the output datum.

**Affected txs:** `submit_reward_yes/no` (would eliminate 3 params each), `sell_position_yes/no` (would eliminate 1 param each)

**Potential fix:** The resolver needs to evaluate reference datum field access at resolve time (fetching the UTxO, decoding the datum, extracting the field value) before building amount expressions. Currently it defers evaluation and the amount builder doesn't know how to handle the unevaluated expression.

---

### 7. Nested / list metadata cannot be emitted — `674` block dropped

> **Re-verified 2026-06-25 against the most recent on-chain txs (trix 0.26.2 / tx3-cardano 0.23.0).**
> This is the global quirk #14, confirmed still open in 0.23 — and it is **the single biggest gap**
> between our user txs and the real ones.

**On-chain reality:** every real Bodega buy / sell / reward tx carries a CIP-20 `674` block, and
batcher txs additionally carry a CIP-25 `721` block. The `674` block is **not cosmetic** — it is
the trade record the Bodega indexer / UI reads. Real shape (buy `9ae49a2a…`, market 262F):

```jsonc
674: {
  "msg":  ["Bodega Market - Buy Position", "FIFA WC | France or Spain wins the World Cup", "Yes"],
  "data": { "id": "262F_FIFA_WC_FRANCE_", "option": 0, "side": "Yes", "action": "Buy Position",
            "address": ["addr1q822…", "00g5r3mg…"], "time": 1782399613363,
            "amount": 226, "asset": "", "price": 429935 },
  "hash": "7bb9c58a8086d1694386a541027bfcef113b846dd95428c19bf82b7f94d86a61"
}
```

`msg` is a `List<String>` and `data` is a nested `Map`.

**Why tx3 can't emit it (source-confirmed, not just observed):**

- `tx3-cardano-0.23.0/src/coercion.rs::expr_into_metadatum` matches **only**
  `tir::Expression::{Number, String, Bytes}` → everything else falls through to
  `CoerceError(_, "Metadatum")`. There is **no `Map` or `List`/`Array` arm**, even though pallas's
  `Metadatum` enum has `Map` and `Array` variants.
- `tx3-cardano-0.23.0/src/compile/mod.rs::compile_auxiliary_data` calls `expr_into_metadatum`
  directly on each `674` value, so a `Map`/`List` value errors out the whole resolution.
- `trix check` and `trix build` **both pass** — the grammar accepts the nested literal and the
  TIR even contains `{"Map": …}` / `{"List": …}` nodes (verified via `trix inspect tir`). The
  failure is at **resolve** time, which is why a build-only check is misleading here.
- Even a single top-level `674 => "string"` would technically resolve, but it does **not** match
  the real shape (a `Map`) and is non-standard CIP-20, so it was not added.

**Decision (2026-06-25):** leave the `674`/`721` metadata **off** and **escalate the tx3 fix** (see
below). The position datum is value-correct and the on-chain validator/batcher ignore metadata, so
funds and shares flow correctly; the only loss is byte-identity and the Bodega-indexer trade record.

**Escalation — tx3 feature request:** add `Map` and `Array` arms to
`tx3-cardano/src/coercion.rs::expr_into_metadatum` (recursively coercing `tir::Expression::Struct`/
record → `Metadatum::Map` and `tir::Expression::List`/`Tuple` → `Metadatum::Array`), so the existing
grammar + TIR support (which already lower nested literals) reaches the resolver. With that one
function fixed, `buy_position` / `sell_position` / `submit_reward` could emit the exact `674` block
and become byte-identical to the Bodega-frontend txs. Tracked in `protocols_tx3/TX3-0.23-UPGRADE.md`
and global memory quirk #14.

**Affected txs:** all user-facing txs (`buy_position`, `sell_position`, `submit_reward`); also blocks
implementing batcher txs' `721` NFT metadata.

---

## ~~Solved~~ Limitations (fixed in recent tx3c releases)

### ~~Reference Inputs Cannot Read Datum Values~~ — SOLVED in tx3c v0.17.0

**Previously:** tx3 could add reference inputs to a transaction but datum contents were opaque — no field access on reference blocks.

**Fixed:** tx3c v0.17.0 (#318) added `datum_is` on reference blocks. Field access works for **datum construction** in outputs. Applied to all 6 user-facing txs to read `outref_id` from the `ProjectInfoDatum` reference — eliminating `project_outref_tx` + `project_outref_idx` as separate params.

**Remaining limitation:** Field access in amount expressions is still not supported (see active limitation #6 above).

### ~~Record Field Name Shadowing~~ — SOLVED in tx3c v0.17.0

**Previously:** tx param or env var names that collided with type field names caused a lowering panic (`not yet implemented` on `RecordField` symbol).

**Fixed:** tx3c v0.17.0 (#316) added support for shadowing of record field names.

---

## Non-Issues / Clarifications

### `admin_fee_percent` Must Remain a Caller Parameter

On-chain analysis (2026-04-14, market 1B60_CRUDE_OIL_CLOSES) shows `pos_admin_fee_percent` in PositionDatum can differ from `ProjectInfoDatum.admin_fee_percent` within the same market (values 200 and 10 observed). The deployed contract (9-field PositionDatum) uses per-position fee values — likely a BODEGA holder discount mechanism. Reading from the ProjectInfoDatum reference would produce incorrect datums for discounted users.

### `collateral_return` / `total_collateral` Not Generated

Real on-chain txs include explicit `collateral_return` (field 16) and `total_collateral` (field 17). The TRP does not generate these. This is cosmetic — transactions still validate.

---

## Summary

| # | Status | Description | Workaround | Impact |
|---|--------|-------------|------------|--------|
| 1 | Active (permanent) | Position total/price need `ln`/`exp` (LMSR) — `*`/`/` (0.22) are NOT enough; formula over-counts the real output (proven vs tx `f57e74aa…`, +894 011) | Caller runs the off-chain LMSR engine | `total_lovelace` + `unit_price` stay params per buy/sell tx |
| 2 | Active | No dynamic input/output lists | None — batcher txs not implementable | 5 txs blocked (all batcher/admin) |
| 3 | Active | No tuple types | N/A (deployed contract avoids tuples) | Low |
| 4 | ~~Active~~ → **Solved (#343, 0.23)** | ~~Enums not passable as params~~ | `candidate: CandidateIdx` param (arg `{"struct":{"constructor":N,"fields":[]}}`) | 6 variants → **3 txs**; needs a resolver ≥0.23 |
| 5 | Active | No conditional logic | Not implemented (no token markets active) | Would need `_ada`/`_token` variants for buy/sell |
| 6 | Active | Ref datum fields only in datum construction | Keep params for amount-used fields | 4 extra params across submit_reward + sell_position |
| 7 | Active (escalated) | Nested/list metadata not coercible (`coercion.rs::expr_into_metadatum` = primitives only; quirk #14, re-verified 0.23) | Drop the `674`/`721` block; fix tx3 to emit it | All user txs lack the CIP-20 trade record; not byte-identical to real txs |
| - | ~~Solved~~ | ~~Reference inputs can't read datums~~ | Fixed in tx3c v0.17.0 (#318) | `outref_id` now read from reference |
| - | ~~Solved~~ | ~~Record field name shadowing panic~~ | Fixed in tx3c v0.17.0 (#316) | No longer need to rename params |

**Transactions blocked by tx3 limitations:** 5 out of 11 total (all batcher/admin operations).
**Enum variant duplication:** RESOLVED — the 6 `_yes`/`_no` variants are now **3 txs** with a
`candidate: CandidateIdx` param (#343, tx3 0.23; requires a resolver ≥0.23 to invoke).
**Pending if token-payment markets appear:** `buy_position` and `sell_position` would need `_ada`/`_token` variants.
