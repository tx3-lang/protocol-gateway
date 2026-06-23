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

### Metadata note (behavior change)

The old txs carried an **inconsistent** CIP-20 `674` message: present on the `_no`
variants and `submit_reward_yes`, absent on `buy_position_yes`/`sell_position_yes`.
A single collapsed tx cannot vary metadata per candidate (no `if`/`when` in tx3), and
these strings are cosmetic labels that do **not** appear in real on-chain Bodega txs.
**The `674` metadata is now dropped from all three txs.** Consequence:
`buy_position`/`sell_position` with `Candidate0` are byte-identical to the old `_yes`
references; `submit_reward` (both candidates) loses its metadata block.

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
