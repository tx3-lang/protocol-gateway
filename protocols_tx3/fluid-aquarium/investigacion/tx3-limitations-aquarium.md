# TX3 Limitations Found During Fluid Aquarium Implementation

Discovered with trix 0.21.1 (2026-04-08).

---

## Bugs (workaround applied, functional)

### 1. Param/Field Name Collision (Lowering Panic)

Same as Indigo limitation #2. Type field names must not collide with tx param names.

**Workaround applied:** Prefixed all type fields: `ct_policy`, `td_tank_owner`, `co_paying_token_idx`, `st_batcher`, etc.

---

## Active Limitations (impact on production use)

### 2. Withdrawal Redeemer Not Generated in Witness Set

**Severity: High — blocks on-chain execution of `consume_oracle`**

The `cardano::withdrawal` block correctly adds the withdrawal entry to the tx body (field 5) with the right credential and 0 ADA amount. However, the corresponding redeemer (purpose=reward, index=0) is **NOT generated** in the witness set (field 5 of witnesses).

Tested with both `redeemer: ()` (unit) and `redeemer: OracleRedeemer { ... }` (typed struct) — neither generates the witness redeemer. The TIR intermediate representation **does** contain the redeemer data correctly (`adhoc[0].data.redeemer`), confirming the bug is in the TIR→CBOR builder, not in parsing.

**On-chain reference:** [`bf7b20eb...`](https://cexplorer.io/tx/bf7b20eb2c69bded174ea2e560f974973d1cede486bd30fcf712f85a40fbf28e)
- Body field 5 (withdrawals): `f11d36e2cb...` = 0 ← correct
- Witness redeemer (3, 0): `OracleRedeemer { PriceDataCharlie { ... } }` ← present on-chain, missing in generated

**Generated tx:**
- Body field 5 (withdrawals): `f11d36e2cb...` = 0 ← correct ✓
- Witness redeemer (3, 0): **ABSENT** ← bug

**Impact:** `consume_oracle` tx cannot execute on-chain. The oracle validator has no redeemer to evaluate.

**Workaround:** None within tx3. The caller must post-process the CBOR to inject the withdrawal redeemer into the witness set before submission.

**Proposed fix for tx3-lang:** When a `cardano::withdrawal` block includes `redeemer:`, generate a corresponding entry in the witness set with purpose=reward and the provided redeemer data. The TIR already has the correct structure — only the CBOR builder needs updating.

### 3. Custom Types Cannot Be Passed as Parameters

Same as quirk #6 in `tx3-quirks.md`. The `Bytes` type wraps values as CBOR ByteString, not raw Plutus Data.

**Impact on Aquarium:**
- `execute_scheduled`: The `batcher` field in the `ScheduledTx` redeemer expects an on-chain `Address` type (Constr with payment/stake credentials). Passing it as `Bytes` produces a ByteString wrapper instead.

**Workaround applied:** Pass `"00"` placeholder for `batcher_addr_cbor`. For production, the caller must construct the full redeemer CBOR externally.

### 4. List Values Cannot Be Passed as Invoke Parameters

`List<T>` works correctly in type definitions and as hardcoded literals (e.g., `[]`). However, list values **cannot be passed as JSON invoke args**. The `from_json()` resolver only supports Int, Bool, Bytes, Address, and UtxoRef.

**Impact on Aquarium:** The `or_signatures` field of `OracleRedeemer` is `List<OracleSignature>`. Since PriceDataCharlie txs on-chain use 0 signatures, the empty list `[]` is hardcoded in the tx3 source. If a variant required non-empty signatures, they could not be passed dynamically.

**Workaround applied:** Hardcoded `[]` in tx source. Works for the PriceDataCharlie case.

### 5. Signers Extracts Payment Key — Some Validators Need Staking Key

`signers { Party, }` works and extracts the **payment key hash** from the party address. This is correct for `consume_oracle` (signer = user payment key) and `withdraw_tank`.

However, `execute_scheduled` and `stake_fldt` require the **staking key hash** as required signer (the Aiken validators extract it from `address.stake_credential`). tx3 has no way to extract the staking key from an address.

**Workaround applied:** `consume_oracle` and `withdraw_tank` use `signers { User, }`. `execute_scheduled` and `stake_fldt` use a raw `signer_hash: Bytes` param with the staking key hash.

**Proposed enhancement:** Support `signers { stake_key_of(Party), }` or similar to extract the staking credential.

### 6. Cannot Read Datum from Reference Inputs

Same as Indigo limitation #4. Reference blocks add UTxOs to `reference_inputs` but tx3 cannot read their datum values.

**Impact on Aquarium:**
- Parameters datum (`DatumParameters`) contains `min_to_stake`, `address_rewards`, `min_ada` — used by validators but not readable by tx3.
- Oracle datums (Charli3 price feeds) contain price/expiration data needed for the oracle redeemer.
- Tank datums contain `allowedTokens`, `tankOwner`, `scheduledAmount` etc.

**Workaround applied:** Tank datum is passed as raw CBOR (`tank_datum_cbor: Bytes`) for creation txs. For consume/withdraw, the datum is propagated via `datum: tank_input` (copying from the consumed input). Oracle values are passed as individual typed params.

### 7. Collateral Requires Pure ADA UTxO

The `collateral {}` block resolves a UTxO from the specified party's address. Cardano requires collateral UTxOs to contain **only ADA** (no native tokens). If the wallet's UTxOs all contain native tokens, collateral resolution fails with "Input not resolved".

**Workaround applied:** Use wallets that have at least one pure-ADA UTxO.

---

## Not Applicable / Non-Issues

### Metadata

On-chain Aquarium txs contain `CBORTag(259, {})` as auxiliary data — an empty CIP-68 metadata tag. Not protocol-specific, not modeled.

### Redeemer Index Differences

Generated redeemer indices may differ from on-chain because input ordering depends on UTxO selection at build time. Expected and correct.

---

## Summary Table

| # | Limitation | Severity | Blocking | Workaround |
|---|-----------|----------|----------|------------|
| 1 | Param/field name collision | Low | No | Prefix type fields |
| 2 | **Withdrawal redeemer not in witness set** | **High** | **Yes** (consume_oracle) | Post-process CBOR externally |
| 3 | Custom types as params (Address in redeemer) | Medium | Partial | Bytes placeholder |
| 4 | List values not passable as invoke args | Low | No | Hardcode in tx source |
| 5 | Signers require raw key hash | Low | No | Pass hash as Bytes param |
| 6 | No ref input datum access | Medium | No | Pass values as params |
| 7 | Collateral needs pure ADA UTxO | Low | No | Use correct wallet |
