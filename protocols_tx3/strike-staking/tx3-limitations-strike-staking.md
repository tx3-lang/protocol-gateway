# TX3 Limitations Found During Strike-Staking Protocol Implementation

Discovered with trix 0.20.0 (2026-03-30).

---

## Bugs (workaround applied, functional)

### 1. ~~Param/Field Name Collision (Lowering Panic)~~ — FIXED

**Fixed in:** trix pre-release — `(lang) Support shadowing of record field names (#316)`

~~When a tx parameter or env var has the same name as a field in any type definition, `trix build` panics with `not yet implemented` in `lowering.rs` on a `RecordField` symbol.~~

**Workarounds removed:** params and env vars now use their original names (`amount`, `staked_at`, `mint_policy_id`) directly — no renaming or aliasing needed.

### 2. ~~Datum Spread on Consumed Inputs Fails at Runtime~~ — FIXED

**Fixed in:** trix pre-release (not explicitly listed in release notes, but confirmed working)

~~The spread syntax (`...current_stake`) to propagate datum fields from a consumed input caused a TRP error: `property index 0 not found in None`.~~

**Workaround removed:** `add_stake` now uses `datum: StakingDatum { ...current_stake }` directly. The `staked_at` param was removed — the datum is propagated automatically from the consumed input.

---

## Active Limitations (impact on production use)

### 3. ~~`slot_to_time()` Returns Seconds — `staked_at` must be a caller param~~ — RESOLVED (tx3 0.23)

**Resolved 2026-06-24** (tx3 0.22 added the `*` operator, #339). The `staked_at` param was **dropped** from `stake`; the datum field is now computed on-chain:

```tx3
staked_at: slot_to_time(tip_slot()) * 1000,
```

`slot_to_time(tip_slot())` resolves to exactly `cursor.timestamp` (the slot delta is zero at the tip). The mainnet TRP populates `cursor.timestamp` in POSIX **seconds**, so we multiply by 1000 to get the milliseconds the Plutus contract expects. The earlier "tx3 lacks multiplication" blocker is **obsolete** — `*` is legal since 0.22.

**Verified by live-resolve (`trp.resolve`, mainnet TRP, 2026-06-24):** without `*1000` the field resolved to `1782314614` (10-digit seconds); with `*1000` it resolves to `1782314668000` (13-digit ms), matching the scale of the real on-chain stake datum (`f70239fa…` → `1773146014192`, also 13-digit ms) and falling inside the validity window (`staked_at_ms ≤ until_slot→ms`, exactly tip+200 slots). `add_stake` already preserved the field via datum spread (#2), so it needed no change.

### 4. `collateral_return` / `total_collateral` Not Generated

Real on-chain transactions include explicit `collateral_return` (field 16) and `total_collateral` (field 17) in the transaction body. The TRP does not generate these fields.

**Impact:** Functional — transactions still validate without these fields. However, the generated CBOR diverges from what wallets and real transaction builders typically produce. This is a cosmetic difference, not a structural error.

---

## Summary

| # | Type | Description | Workaround | Params added |
|---|------|-------------|------------|--------------|
| 1 | ~~Bug~~ | ~~Param/field name collision~~ | **FIXED** (#316) — workarounds removed | 0 |
| 2 | ~~Bug~~ | ~~Datum spread fails at runtime~~ | **FIXED** — spread works, param removed | 0 |
| 3 | ~~Limitation~~ | ~~`slot_to_time()` returns seconds, not ms~~ | **RESOLVED** (tx3 0.22 `*`) — `staked_at: slot_to_time(tip_slot()) * 1000`, param dropped | 0 |
| 4 | Limitation | No `collateral_return`/`total_collateral` | None needed (cosmetic) | 0 |

**Total extra params due to limitations:** 0 (the `staked_at` param was eliminated 2026-06-24 — computed on-chain via `slot_to_time(tip_slot()) * 1000`).
