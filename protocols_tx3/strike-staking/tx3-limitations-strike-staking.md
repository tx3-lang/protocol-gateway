# TX3 Limitations Found During Strike-Staking Protocol Implementation

Discovered with trix 0.20.0 (2026-03-30).

---

## Bugs (workaround applied, functional)

### 1. Param/Field Name Collision (Lowering Panic)

When a tx parameter or env var has the same name as a field in any type definition, `trix build` panics with `not yet implemented` in `lowering.rs` on a `RecordField` symbol.

**Affected names:**
- `amount` — tx param AND field of `MintRedeemer.Mint`
- `staked_at` — tx param AND field of `StakingDatum`
- `mint_policy_id` — env var AND field of `StakingDatum`

**Workaround applied:**
- `amount` renamed to `stake_amount`
- `staked_at` renamed to `staked_at_time`
- `mint_policy_id` aliased via `locals { credential_policy: mint_policy_id, }` to avoid collision

**Fix for tx3-lang:** Prioritize param/env scope over type field scope in symbol resolution.

### 2. Datum Spread on Consumed Inputs Fails at Runtime

The spread syntax (`...current_stake`) to propagate datum fields from a consumed input caused a TRP error: `property index 0 not found in None`.

**Affected tx:** `add_stake` — needed to preserve the existing `StakingDatum` while adding more STRIKE.

**Workaround applied:** Replaced spread with explicit datum field assignment and added `staked_at_time` as a tx parameter so the caller provides the original datum value.

```tx3
// WANTED (fails at runtime):
output {
    to: StakingScript,
    amount: current_stake + extra_strike,
    datum: StakingDatum { ...current_stake }
}

// WORKAROUND (works):
output {
    to: StakingScript,
    amount: current_stake + extra_strike,
    datum: StakingDatum {
        owner_address_hash: owner_pkh,
        staked_at: staked_at_time,           // passed as param
        mint_policy_id: credential_policy,   // aliased env var
    },
}
```

**Impact:** The API caller must query the existing datum off-chain and pass `staked_at_time` explicitly. Without the bug, the datum would be automatically propagated.

**Fix for tx3-lang:** Ensure datum values from consumed inputs are available for spread in output datum constructors.

---

## Active Limitations (impact on production use)

### 3. `slot_to_time()` Returns Seconds Instead of Milliseconds

The `staked_at` datum field requires POSIX time in milliseconds (as used on-chain by Plutus). `slot_to_time()` exists but returns seconds, making it unusable directly.

**Impact:** The API caller must compute and pass the POSIX timestamp in milliseconds. The commented-out line in `main.tx3` shows the intent:

```tx3
staked_at: staked_at_time, // (slot_to_time(tip_slot()) + 200),
```

**If tx3 fixed `slot_to_time()` to return milliseconds (or added a `slot_to_time_ms()` variant), the `staked_at_time` param could be eliminated from `stake` and `add_stake`.** Since tx3 lacks multiplication, even with the seconds value we can't do `slot_to_time(tip_slot()) * 1000`.

### 4. `collateral_return` / `total_collateral` Not Generated

Real on-chain transactions include explicit `collateral_return` (field 16) and `total_collateral` (field 17) in the transaction body. The TRP does not generate these fields.

**Impact:** Functional — transactions still validate without these fields. However, the generated CBOR diverges from what wallets and real transaction builders typically produce. This is a cosmetic difference, not a structural error.

---

## Summary

| # | Type | Description | Workaround | Params added |
|---|------|-------------|------------|--------------|
| 1 | Bug | Param/field name collision | Rename params + alias env var | 0 |
| 2 | Bug | Datum spread fails at runtime | Explicit fields + extra param | +1 (`staked_at_time` in `add_stake`) |
| 3 | Limitation | `slot_to_time()` returns seconds, not ms | Caller computes POSIX time in ms | +1 (`staked_at_time` in `stake`) |
| 4 | Limitation | No `collateral_return`/`total_collateral` | None needed (cosmetic) | 0 |

**Total extra params due to limitations:** 2 (both `staked_at_time`, could be eliminated with fixes #2 and #3).
