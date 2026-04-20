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

### 3. `slot_to_time()` Returns Seconds Instead of Milliseconds

The `staked_at` datum field requires POSIX time in milliseconds (as used on-chain by Plutus). `slot_to_time()` exists but returns seconds, making it unusable directly.

**Impact:** The API caller must compute and pass the POSIX timestamp in milliseconds. The commented-out line in `main.tx3` shows the intent:

```tx3
staked_at: staked_at_time, // (slot_to_time(tip_slot()) + 200),
```

**If tx3 fixed `slot_to_time()` to return milliseconds (or added a `slot_to_time_ms()` variant), the `staked_at` param could be eliminated from `stake`.** Since tx3 lacks multiplication, even with the seconds value we can't do `slot_to_time(tip_slot()) * 1000`. Note: `add_stake` no longer needs this param thanks to datum spread fix (#2).

### 4. `collateral_return` / `total_collateral` Not Generated

Real on-chain transactions include explicit `collateral_return` (field 16) and `total_collateral` (field 17) in the transaction body. The TRP does not generate these fields.

**Impact:** Functional — transactions still validate without these fields. However, the generated CBOR diverges from what wallets and real transaction builders typically produce. This is a cosmetic difference, not a structural error.

---

## Summary

| # | Type | Description | Workaround | Params added |
|---|------|-------------|------------|--------------|
| 1 | ~~Bug~~ | ~~Param/field name collision~~ | **FIXED** (#316) — workarounds removed | 0 |
| 2 | ~~Bug~~ | ~~Datum spread fails at runtime~~ | **FIXED** — spread works, param removed | 0 |
| 3 | Limitation | `slot_to_time()` returns seconds, not ms | Caller computes POSIX time in ms | +1 (`staked_at` in `stake`) |
| 4 | Limitation | No `collateral_return`/`total_collateral` | None needed (cosmetic) | 0 |

**Total extra params due to limitations:** 1 (`staked_at` in `stake`, could be eliminated with fix #3).
