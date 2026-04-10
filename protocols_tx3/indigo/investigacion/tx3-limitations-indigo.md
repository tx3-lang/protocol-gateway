# TX3 Limitations Found During Indigo Protocol Implementation

Discovered with trix 0.21.1 (2026-04-07).

---

## Bugs (workaround applied, functional)

### 1. Type Names Starting with Primitive Keywords (Parser)

Custom type names starting with `Int`, `Bool`, `Bytes`, `Address`, `UtxoRef`, or `AnyAsset` cause parse failures. PEG parser matches primitive keyword greedily.

**Example:** `InterestData` → parser matches `Int`, fails on `erestData`.
**Workaround applied:** Renamed to `CdpInterest`.
**Fix for tx3-lang:** `("Int" | ...) ~ !ASCII_ALPHANUMERIC`

### 2. Param/Field Name Collision (Lowering Panic)

When a tx param has the same name as a type field, lowering panics with `not yet implemented`.

**Example:** Param `owner_pkh` collides with `CDPCreatorRedeemer::CreateCDP { owner_pkh }`.
**Workaround applied:** Prefixed type fields: `cr_owner`, `ci_timestamp`.
**Fix for tx3-lang:** Prioritize param scope over type field scope.

---

## Active Limitations (impact on production use)

### 3. No Multiplication/Division

Only `+` and `-` supported. Cannot compute collateral ratios, interest rates, or prices.

**Impact:** All computed values must be pre-calculated by the API caller before invoking.
**Affected params:** Collateral ratios, protocol fees, price calculations.

### 4. Cannot Read Datum from Reference Inputs

`reference` blocks add UTxOs to `reference_inputs` but tx3 cannot read their datum values.

**Impact:** Oracle prices, interest accumulator, and SP pool snapshots live in UTxO datums that are already reference inputs, but their values can't be extracted in tx3. The API caller must query them off-chain (Koios/Ogmios) and pass as params.

**If tx3 added `reference name { ref: ..., datum_is: Type }` with field access, these params would be eliminated:**

From oracle UTxO (`oracle_data`, type `OracleDatum`):

| Current param | Would become | Used in |
|---|---|---|
| `interest_accumulator` | `oracle_data.od_nonce` or derived value | create_cdp |
| `accumulator` | Same | adjust_cdp_mint, adjust_cdp_burn |
| `timestamp_ms` | `oracle_data.od_expiration` | All CDP txs |

From SP pool UTxO (`sp_input`, type `StabilityDatum::PoolState`):

| Current param | Would become | Used in |
|---|---|---|
| `sp_snapshot_p` | `sp_input.sp_content.sp_snapshot.snapshot_p` | create_sp, close_cdp |
| `sp_snapshot_d` | `sp_input.sp_content.sp_snapshot.snapshot_d` | " |
| `sp_snapshot_s` | `sp_input.sp_content.sp_snapshot.snapshot_s` | " |
| `sp_snapshot_epoch` | `sp_input.sp_content.sp_snapshot.snapshot_epoch` | " |
| `sp_snapshot_scale` | `sp_input.sp_content.sp_snapshot.snapshot_scale` | " |

From SP account UTxO (`account_input`, type `StabilityDatum::Account`):

| Current param | Would become | Used in |
|---|---|---|
| `owner_pkh` | `account_input.acc_content.acc_owner` | adjust_sp, close_sp |
| `iasset_name` | `account_input.acc_content.acc_iasset` | " |
| `acc_snapshot_p/d/s/epoch/scale` | `account_input.acc_content.acc_snapshot.*` | " |

**Total: ~20 params could be eliminated**, reducing SP txs from 10-11 params to 3-4.

**Proposed syntax for tx3-lang:**
```tx3
reference oracle_data {
    ref: oracle_utxo,
    datum_is: OracleDatum,   // ← new: parse datum and expose fields
}
// Then: oracle_data.od_price.ocd_value would work
```

### 5. Datum Field Access on Variant Types

Field access (`input.field`) works on record types but fails on variant types with "not in scope".
Spread (`...input`) inside a variant constructor does work.

**Impact:** SP account datums (`StabilityDatum::Account`) require all fields as explicit params instead of reading from input datum.

**Works:** `...position_input` inside `StakingDatum::StakingPosition { ... }`
**Fails:** `account_input.acc_content.acc_owner` on `StabilityDatum` (variant)

---

## Future Limitations (not blocking today, will block on upgrade)

### 6. No Tuple Types

Cannot represent `Map<Int, (Int, POSIXTime)>`.

**Impact today:** None — on-chain staking uses empty datums `Constr(0, [])`.
**Impact on VX staking upgrade:** Will block modeling `StakingPositionContent.lockedAmount` field when Indigo activates VX staking validators with full datum fields.

**Proposed fix:** Add tuple support or allow named records as Map values.
