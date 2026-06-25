# Bodega Market: CBOR Comparison Report

Updated 2026-06-22 for the **tx3 0.23 upgrade** (toolchain: trix 0.26.2, tii spec
`v1beta0`). Compares transactions built by `trix invoke --profile mainnet --skip-submit`
against the previously verified references (market CC01_ADA_REACHES_060_).

## What changed in this revision (tx3 0.23)

**One** source-level improvement landed; the second was attempted and **reverted** after a
real-tx check (see "Inline fee — reverted" below).

1. **6 `_yes`/`_no` txs collapsed into 3** ✅ — taking `candidate: CandidateIdx` as an enum
   parameter (self-describing args, PR #343):
   - `buy_position_yes` + `buy_position_no`   → **`buy_position`**
   - `submit_reward_yes` + `submit_reward_no` → **`submit_reward`**
   - `sell_position_yes` + `sell_position_no` → **`sell_position`**

   `pos_candidate` is now driven by the `candidate` param instead of a hardcoded
   `CandidateIdx::Candidate0/1` literal. `Candidate0` → `Constr(0)` (YES),
   `Candidate1` → `Constr(1)` (NO) — identical to the old per-variant literals. Confirmed
   against a real on-chain NO buy (tx `f57e74aa…`, market 5637_SPCX): `candidate=Constr(1)`.

2. **`total_lovelace` inline — INVESTIGATED, DISCARDED.** ❌ The **real** deployed formula
   (LMSR contract, reverse-engineered + verified vs on-chain — see
   `tx3-limitations-bodega-market.md §1`) is:

   ```
   total = LMSR_cost + floor(LMSR_cost*(pi_admin% + pos_admin%)/10000) + batcher + envelope
   ```

   Verified exact on tx `f57e74aa…` (5637_SPCX NO):
   `147 149 730 + 5 885 989 + 700 000 + 2 000 000 = 155 735 719` ✓ (both admin% = 200 → 4%).
   The old `main.tx3` polynomial put the admin fee on `buy_amount` (= 6 780 000) instead of on
   `LMSR_cost` (= 5 885 989) → that was the +894 011 error. Not inlinable because **(a)**
   `LMSR_cost ≠ buy_amount*unit_price` — `unit_price` is a rounded average; the real cost needs
   ln/exp off-chain (off by up to ~100k lovelace in other trades), and **(b)** even passing
   `LMSR_cost` as a param saves no compute, needs `pi_admin%` as an extra param (unreadable
   from the ref datum in amounts, quirk #13), and the closed-source rate combo is only
   reverse-engineered. So `total_lovelace` + `unit_price` **stay caller params**.

### Metadata note (behavior change) — CORRECTED 2026-06-25

> ⚠️ **An earlier revision of this note claimed the `674` message was a "cosmetic label that
> does not appear in real on-chain Bodega txs." That is wrong.** A re-check against the most
> recent on-chain txs (June 2026, see "Re-verification" below) shows **every** real
> buy / sell / reward tx carries a rich CIP-20 `674` block, and batcher txs additionally carry
> a CIP-25 `721` block. The metadata is **not** cosmetic — it encodes the trade record the
> Bodega indexer / UI reads.

**The `674` metadata is dropped from all three txs**, but not because it's unimportant — it's
because **tx3 0.23 cannot emit it.** Real shape (from buy `9ae49a2a…`, market 262F):

```jsonc
674: {
  "msg":  ["Bodega Market - Buy Position", "FIFA WC | France or Spain wins the World Cup", "Yes"],
  "data": { "id": "262F_FIFA_WC_FRANCE_", "option": 0, "side": "Yes", "action": "Buy Position",
            "address": ["addr1q822…(≤64B chunk)", "00g5r3mg…(chunk 2)"],   // bech32 split at 64B
            "time": 1782399613363, "amount": 226, "asset": "", "price": 429935 },
  "hash": "7bb9c58a8086d1694386a541027bfcef113b846dd95428c19bf82b7f94d86a61"
}
```

`msg` is a List and `data` is a nested Map. **Verified in the resolver source** that neither can
be coerced: `tx3-cardano-0.23.0/src/coercion.rs::expr_into_metadatum` matches only
`Number`/`String`/`Bytes` and falls through to `CoerceError(_, "Metadatum")` for `Map`/`List`;
`compile/mod.rs::compile_auxiliary_data` calls it on each `674` value. `trix check`/`trix build`
**pass** (they only lower to TIR — the TIR even shows `Map`/`List` nodes), but resolution fails.
So this is **quirk #14, confirmed still open in 0.23**, not a design choice. See
`tx3-limitations-bodega-market.md §6`.

Consequence: our `buy_position`/`sell_position`/`submit_reward` produce a **value-correct
position datum but no `674`/`721` auxiliary data** (and therefore no `auxiliary_data_hash` in the
body). On-chain validity is unaffected (position creation runs no validator and the batcher reads
the *datum*, not the metadata), but the tx is **not byte-identical** to a real Bodega tx and the
trade will not carry the Bodega-indexer record. Restoring it requires either a tx3 fix
(escalated — see limitations doc) or off-chain metadata injection before signing.

## Enum argument format (important)

`CandidateIdx` is passed in the **tagged self-describing form** the resolver's
`from_json` expects (`tx3-resolver/src/interop.rs` — bare values are only accepted for
scalars; aggregates must arrive tagged):

```json
"candidate": { "struct": { "constructor": 0, "fields": [] } }   // Candidate0 = YES
"candidate": { "struct": { "constructor": 1, "fields": [] } }   // Candidate1 = NO
```

> The `{ "Candidate0": {} }` shape that appears under `components.schemas` in the TII is
> the JSON-Schema *documentation* form, **not** the resolver wire form. Passing it bare
> would fall through to `coerce_bare`, which rejects the `Custom` type.

## Transaction params — before vs after

| tx | before (params) | after (params) |
|----|------------------|----------------|
| `buy_position`   | user_pkh, user_stake_key, project_info_ref, buy_amount, batcher_fee_amount, admin_fee_percent, unit_price, total_lovelace | user_pkh, user_stake_key, project_info_ref, **candidate**, buy_amount, batcher_fee_amount, admin_fee_percent, unit_price, total_lovelace |
| `submit_reward`  | user_pkh, user_stake_key, project_info_ref, share_policy_id, candidate_name, envelope_amount, share_amount, batcher_fee_amount | user_pkh, user_stake_key, project_info_ref, **candidate**, share_policy_id, candidate_name, envelope_amount, share_amount, batcher_fee_amount |
| `sell_position`  | user_pkh, user_stake_key, project_info_ref, envelope_amount, share_amount, batcher_fee_amount, admin_fee_percent, unit_price | user_pkh, user_stake_key, project_info_ref, **candidate**, envelope_amount, share_amount, batcher_fee_amount, admin_fee_percent, unit_price |

`submit_reward` keeps **both** `candidate` (the datum index) and `candidate_name` (the
share-token asset name, e.g. `B_CC01_YES` = `425f434330315f594553`). The token name
cannot be derived from the enum without a conditional, and `ProjectInfoDatum`'s
`candidate_*_name` fields can't be used in asset expressions (quirk #13, still open).

## Verification

`trix invoke` requires an interactive TTY (the CShell wallet), so the final byte-for-byte
CBOR diff must be run by a human (see "Reproduce" below). The structural verification
done here is **deterministic and toolchain-checked** via the compiled TIR
(`trix inspect tir --tx <name> --pretty`), which fixes the datum constructors, field
order, param wiring, amount arithmetic, and metadata — everything that determines the
CBOR except the resolved leaf values.

**TIR diff of each new tx against its old `_yes` baseline — only the intended deltas:**

| tx | TIR delta vs old `_yes` |
|----|--------------------------|
| `buy_position`  | `pos_candidate` `Struct{constructor:0}` → `EvalParam[candidate, Custom:CandidateIdx]` (only — `total_lovelace` stays a param) |
| `sell_position` | `pos_candidate` `Struct{constructor:0}` → `EvalParam[candidate, Custom:CandidateIdx]` (only) |
| `submit_reward` | metadata `[674:"…Yes"]` → `[]`; `pos_candidate` `Struct{constructor:0}` → `EvalParam[candidate, Custom:CandidateIdx]` |

### On-chain resolve — what's confirmed (updated local Dolos, resolver ≥0.23)

`buy_position` (Candidate0/YES, CC01) resolved end-to-end; the position datum decodes with
the right structure and `pos_candidate = Constr(0)` from the
`{"struct":{"constructor":0,"fields":[]}}` arg — **the enum collapse (task 1) works.** The
output amount equals the `total_lovelace` param (now caller-supplied, not inlined).

Cross-checked against the **real on-chain NO buy** (tx `f57e74aa…`, 5637_SPCX). Decoded
position datum, field by field:

| field | real f57e74aa | our `buy_position(Candidate1, …)` |
|-------|---------------|-----------------------------------|
| outref_id | `6d61b61b…#3` | `6d61b61b…#3` (read from ProjectInfoDatum `1ac43064…#0`) ✓ |
| user_pkh / stake | `012691fb…` / `SomePkh(75692f73…)` | same (params) ✓ |
| pos_type | Constr(0) BuyPos | Constr(0) ✓ |
| pos_amount / batcher / admin% / unit_price | 339 / 700000 / 200 / 434070 | same (params) ✓ |
| **pos_candidate** | **Constr(1)** = NO | **Constr(1)** (from `{"struct":{"constructor":1,"fields":[]}}`) ✓ |
| output amount | 155 735 719 | = `total_lovelace` param 155 735 719 ✓ |

→ fixture `buy_position_real_no.json` reproduces it. **Values & structure match.**

### Datum is value-equivalent but NOT byte-identical (encoding convention)

tx3's resolver serializes Plutus `Constr` fields as **definite-length** CBOR arrays; the
on-chain Aiken contract uses **indefinite-length** (`9f…ff`):

```
our tx3 (CC01 resolve):  outer d87989(def 9)  outref d87982(def 2)  SomePkh d87981(def 1)  empty d87980
on-chain (f57e74aa):     outer d8799f…ff(indef)  outref d8799f…ff      SomePkh d8799f…ff      empty d87980
```

Same Plutus data, same decoded values → the validator/batcher accepts it identically, but
the raw datum bytes (and any datum hash) differ. This is a tx3-resolver convention, **not**
introduced by this change — the original protocol emitted the same definite-length form.
So "byte-for-byte identical to the on-chain datum" is **not achievable via tx3** for these
Aiken datums; value-level equivalence is.

### Resolver-version requirement (critical for deployment)

The shipped change (enum param, #343) needs a **resolver on tx3 ≥0.23**. Observed while
debugging:

| Resolver | inline `*`/`/` (0.22) | enum param `#343` (0.23) |
|----------|------------------------|---------------------------|
| `--profile mainnet` TRP (`trp-m1.demeter.run`) | ❌ `unknown variant Mul` (it is <0.22) | ❌ |
| stock local Dolos ~0.22 | ✅ | ❌ `target type not supported: Custom("CandidateIdx")` |
| updated local Dolos (≥0.23) | ✅ | ✅ |

The trix 0.26 compiler runs ahead of the deployed resolvers. **This needs the TRP/resolver
on tx3 ≥0.23** — the demeter `trp-m1` endpoint used by the built-in `mainnet` profile was
too old at the time (the mainnet TRP is ≥0.23 as of 2026-06-23). (This was also the trail
that exposed the inline-fee bug: getting the resolver new enough to even run it.)

### Still to confirm (human step)

- **Deployment:** bodega is the first protocol in `protocols/` to ship a
  `components.schemas` section (the `CandidateIdx` enum). Confirm the API server
  (tx3-sdk 0.9.2) serves the `candidate` param / OpenRPC correctly after the `.tii` swap,
  and that production resolves against a tx3 ≥0.23 backend.

## Re-verification against the most recent on-chain txs (2026-06-25)

Re-fetched live activity on the active deployment (Instance B; blocks ~13.59M) and decoded the
CBOR of one real tx of each user type. **All datums match in value and field count.** The only
divergences are the metadata gap (above), the CBOR length convention, and a reference-input the
real txs don't carry (see below).

### Reference txs used (fresh)

| Our tx | Real on-chain tx | Market | Block | What it confirms |
|--------|------------------|--------|-------|------------------|
| `buy_position`   | `9ae49a2a3581859b8d039d38183242a0386fdf5e012f9530f93349b4173627c3` | 262F (FIFA WC) | 13596176 | BuyPos datum, 9 fields |
| `submit_reward`  | `fba6e8625299ebcec424188c8c34ddb9e0a0ca316d41d45ea9c99291c4764d4f` | 551C | — | RewardPos datum, 9 fields |
| `create_market`  | `79fd868d5dca27994cf3521b4716611ed0bf739d59208a8d72923b6106675efb` | 3CE5 (FIFA WC OU) | 13593521 | ProjectInfo 17 + Prediction 7 fields, mint, output order |
| (batcher, N/A)   | `5879cee3de0dcca4ec30414d6ba92f53690b4c80acfefe34906a0ee06ccdfdff` | 551C | — | reward batch that consumed the `submit_reward` position |

(`sell_position` / RefundPos = `Constr(1)` was not seen in the current window — sells are rare
right now — but is unchanged from the prior verification and follows by elimination from
BuyPos=`Constr(0)` and RewardPos=`Constr(2)` both reconfirmed here.)

### Position datum — decoded field-by-field (real buy `9ae49a2a…`)

| # | field | real value | our datum source | match |
|---|-------|-----------|------------------|-------|
| 0 | outref_id | `c804779a…#3` | `project_info.outref_id` (ref input) | value ✓ |
| 1 | pos_user_pkh | `d4a30993…` | param | ✓ |
| 2 | pos_user_stake_key | `SomePkh(68cfb978…)` | param | ✓ |
| 3 | pos_type | `Constr(0)` BuyPos | literal | ✓ |
| 4 | pos_amount | 226 | param `buy_amount` | ✓ |
| 5 | pos_batcher_fee | 700000 | param | ✓ |
| 6 | pos_admin_fee_percent | 200 | param | ✓ |
| 7 | pos_unit_price | 429935 | param | ✓ |
| 8 | pos_candidate | `Constr(0)` YES | param `candidate` | ✓ |

Reward (`fba6e862…`) decoded identically with `pos_type = Constr(2)` (RewardPos), `admin% = 0`,
`unit_price = 0`, `pos_candidate = Constr(0)` — exactly what `submit_reward` emits. This
**reconfirms the deployed Reward/Refund swap** (`Constr(2) = RewardPos`, not the GitHub-v2 order).

### create_market — structure reconfirmed

Real raw-CBOR output order is `[0] ProjectInfo · [1] Prediction · [2] Treasury(open_fee) · [3]
change` — **identical to our tx3 output order.** Mint = `{PROJECT_INFO_NFT:1,
PROJECT_PREDICTION_NFT:1}`. Datums: ProjectInfoDatum **17 fields**, PredictionDatum **7 fields**
(both match our types). The real mint redeemer is `Mint(seed_outref, 0, 0, 1, 2)` →
`info_out_idx=0, [field3]=0, settings_ref_idx=1, treasury_out_idx=2`. Our redeemer takes these as
params; `create_market.json` must supply values consistent with our (matching) output order. The
3rd field (we name it `pred_out_idx`) is `0` on-chain even though the pred NFT sits at output 1 —
its exact meaning stays unconfirmed (closed-source policy); it is a caller param regardless.

### Structural divergences (beyond the datum)

| aspect | real on-chain | our tx3 | status |
|--------|---------------|---------|--------|
| CIP-20 `674` / CIP-25 `721` metadata | present (rich) | absent | **blocked** (quirk #14, see above) |
| reference inputs (user txs) | **0** — `outref_id` hardcoded by the backend | 1 ref input to ProjectInfoScript to read `outref_id` | **kept on purpose** — guarantees a correct `outref_id` (a wrong one makes the position unprocessable). Functionally valid; not byte-identical. Decision 2026-06-25: do not trade this safety for fidelity. |
| validity interval (buy) | `[since, until]` ~360-slot window | `until = tip+600` only | minor; no validator on position creation |
| validity interval (reward) | none; instead a `required_signer = user_pkh` | `until = tip+600`, no required signer | minor; harmless |
| Constr CBOR length | indefinite `d8799f…ff` | definite `d87989…` | resolver convention; value-equivalent |

None of these affect on-chain acceptance of the position (creation runs no validator; the batcher
reads the datum). They do mean our user txs are **value-precise but not byte-identical** to a
Bodega-frontend tx — the gap is the metadata + the ref-input + the CBOR length convention.

## Test market: CC01_ADA_REACHES_060_

| Config | Value |
|--------|-------|
| ProjectInfo UTxO | `fc914f41696c345b1a782e53ef6117c90aee1d7561d4442574a1380d40df71c3#0` |
| PositionScript | `addr1w9jw5wpd06f5v53sltrvxpkymraugehamf86r5z3vyl9jygxlhyt4` |
| share_policy_id | `6e8181d047370418d7ef48f013ffa1bd986388e84cd9c6eec676d98e` |
| admin_fee_percent | 200 |
| envelope_amount | 2,000,000 |
| Candidate YES | `B_CC01_YES` (`425f434330315f594553`) |
| Candidate NO | `B_CC01_NO` (`425f434330315f4e4f`) |

> **Liveness (checked 2026-06-25):** the CC01 ProjectInfo UTxO `fc914f41…#0` is **still
> unspent**, so the existing `buy_position.json` / `sell_position.json` / `submit_reward.json`
> fixtures still resolve as-is — no need to regenerate them for a live market. If CC01 ever
> closes, swap to a currently-active market, e.g. **262F** (the one re-verified above): its
> ProjectInfoDatum is read by the same `project_info_ref` mechanism — just point
> `project_info_ref` at 262F's ProjectInfo UTxO and update `share_policy_id` /
> `candidate_name` (`B_262F_YES` / `B_262F_NO`).

## Reproduce (requires a TTY + a tx3 ≥0.23 resolver, e.g. an updated local Dolos)

```bash
cd protocols_tx3/bodega-market
trix check && trix build      # then: cp .tx3/tii/.../main.tii ../../protocols/bodega_market.tii

# Use a ≥0.23 resolver. The built-in `mainnet` TRP (trp-m1.demeter.run) is too old;
# point at an updated local Dolos instead (its profile must use mainnet network params).

# CC01, YES (Candidate0)
trix invoke --skip-submit --args-json-path invoke-args/buy_position.json
trix invoke --skip-submit --args-json-path invoke-args/submit_reward.json
trix invoke --skip-submit --args-json-path invoke-args/sell_position.json

# Real on-chain NO buy (market 5637_SPCX) — compare position out vs tx f57e74aa…
trix invoke --skip-submit --args-json-path invoke-args/buy_position_real_no.json

# create_market (unchanged)
trix invoke --skip-submit --args-json-path invoke-args/create_market.json
```
