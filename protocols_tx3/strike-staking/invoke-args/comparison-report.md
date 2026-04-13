# Strike-Staking: CBOR Comparison Report

Generated 2026-04-13 (updated). Compares transactions built by `trix invoke --profile mainnet --skip-submit` against real on-chain transactions from the active Strike Finance staking contract (`497a8b0085517f1c9065cf3006af4c266454b39c6fa32a9d116c75ee`).

## Reference Transactions (real on-chain)

| Operation | Tx Hash | Block |
|-----------|---------|-------|
| stake | `f70239fa91bcb0df496c011465e440e8cd97955231df956c7de3820e1c861a80` | 13140934 |
| add_stake | `939737ecd4f1ed2cab613a63606802287e79b0525767ddaff6057bbafd28bfab` | 13140518 |
| withdraw_stake | `60f83cf13c421d33a039cd902b57dd67fec4f4afbcdbade5a597b3f816d58b06` | 13140491 |

## Generated Transactions

| Operation | Tx Hash (unsigned) | Args File |
|-----------|--------------------|-----------|
| stake | `9e72ffd4409c59403905092ce8184f1932bac3f77dad2573d0db527f5ac7b536` | `stake.json` |
| add_stake | `4728e6e53f7eb2c586daab2e89fd9a7862b4213383c855d045ba3efbdc7ee9e1` | `add_stake.json` |
| withdraw_stake | `0af5af139784d96c66da5803f92ed83c0b9eefc98832342a997324e7297ea495` | `withdraw_stake.json` |

## Test Wallet

All invocations used a real mainnet wallet that holds the required assets:

- **Address:** `addr1q8gq48ux6xp3p34r58t2pgv4u0lgc0uxulhs9n9k843z0eey3y5c5t4s9eedc645ql56qar2jtezrutfrtz4tuz2zz0sl7dkak`
- **Payment credential:** `d00a9f86d18310c6a3a1d6a0a195e3fe8c3f86e7ef02ccb63d6227e7`
- **Assets:** ~15,230 STRIKE + ~95,772 ADA in wallet
- **Staking UTxO (at script):** `8530244324dfbe1dda2ea12c83025fff1462ad6ec658bf33e8a54d78c627213f#16` (~19,013 STRIKE staked)

## Structural Comparison

### stake

| Field | Real | Ours | Match |
|-------|------|------|-------|
| Body fields (structural) | inputs, outputs, fee, ttl, mint, script_data_hash, collateral, required_signers, collateral_return, total_collateral | inputs, outputs, fee, ttl, mint, script_data_hash, collateral, required_signers, network_id, reference_inputs | see notes |
| Output count | 2 (script + change) | 2 (script + change) | YES |
| Output[0] destination | staking script address | staking script address | YES |
| Output[0] tokens | tracker(1) + owner_nft(1) + STRIKE(staked_amount) | tracker(1) + owner_nft(1) + STRIKE(staked_amount) | YES |
| Output[0] datum | StakingDatum (3 fields) | StakingDatum (3 fields) | YES |
| Output[1] destination | staker wallet | staker wallet | YES |
| Output[1] tokens | change STRIKE | change STRIKE | YES |
| Mint | tracker(+1) + owner_nft(+1) | tracker(+1) + owner_nft(+1) | YES |
| Mint redeemer | `Constr(0, [amount])` | `Constr(0, [amount])` | YES |
| Required signers | 1 (owner_pkh) | 1 (owner_pkh) | YES |
| Collateral | 1 input | 1 input | YES |

### add_stake

| Field | Real | Ours | Match |
|-------|------|------|-------|
| Body fields (structural) | inputs, outputs, fee, script_data_hash, collateral, required_signers, collateral_return, total_collateral, reference_inputs | inputs, outputs, fee, script_data_hash, collateral, required_signers, network_id, reference_inputs | see notes |
| Output count | 2 (script + change) | 2 (script + change) | YES |
| Output[0] destination | staking script address | staking script address | YES |
| Output[0] datum | StakingDatum preserved | StakingDatum preserved | YES |
| Mint | NONE | NONE | YES |
| Spend redeemer | `Constr(0, [])` (AddStakeOrConsumeStakingRewards) | `Constr(0, [])` (AddStakeOrConsumeStakingRewards) | YES |
| Reference inputs | 1 (`486c6c...#0`) | 1 (`486c6c...#0`) | YES |
| Required signers | 1 (owner_pkh) | 1 (owner_pkh) | YES |

### withdraw_stake

| Field | Real | Ours | Match |
|-------|------|------|-------|
| Body fields (structural) | inputs, outputs, fee, mint, script_data_hash, collateral, required_signers, collateral_return, total_collateral, reference_inputs | inputs, outputs, fee, mint, script_data_hash, collateral, required_signers, network_id, reference_inputs | see notes |
| Output count | 1 (all to staker) | 1 (all to staker) | YES |
| Output[0] tokens | STRIKE (full amount returned) | STRIKE (full amount returned) | YES |
| Mint (burn) | tracker(-1) + owner_nft(-1) | tracker(-1) + owner_nft(-1) | YES |
| Spend redeemer | `Constr(1, [])` (WithdrawStake) | `Constr(1, [])` (WithdrawStake) | YES |
| Mint redeemer | `Constr(1, [owner_pkh])` (Burn) | `Constr(1, [owner_pkh])` (Burn) | YES |
| Reference inputs | 1 (`486c6c...#0`) | 1 (`486c6c...#0`) | YES |
| Required signers | 1 (owner_pkh) | 1 (owner_pkh) | YES |
| Redeemer count | 2 (spend + mint) | 2 (spend + mint) | YES |

## Expected Differences (not errors)

These differences are inherent to how `trix invoke --skip-submit` works and do not indicate structural problems:

1. **`vkey_witnesses` absent in ours** — Transactions built with `--skip-submit` are unsigned. Real transactions include witness signatures. This is expected.

2. **`collateral_return` / `total_collateral` (fields 16/17) absent in ours** — Real transactions include explicit collateral return outputs. This is not yet implemented in the TRP but will be supported in a future version.

3. **`network_id` (field 15) present in ours, absent in real** — Optional field added by the TRP. Has no effect on transaction validity.

4. **Input count differs** — We use different UTxOs than the real transactions (different wallet, different time). The coin selection naturally produces different input sets.

5. **Fee values differ** — Our transactions use a placeholder fee (~1.7 ADA) since the script is not evaluated in skip-submit mode. Real transactions have exact fees calculated after Plutus evaluation.

6. **Reference inputs in stake** — Real stake tx uses inline script (fields 16/17), ours uses reference script input (`486c6c...#0`). Both are valid approaches; using reference scripts is more efficient.

## Redeemer Summary

All redeemers match the on-chain contract exactly:

| Operation | Type | Constructor | Fields | On-chain occurrences |
|-----------|------|-------------|--------|---------------------|
| stake | mint | `Constr(0, [Int])` | amount | 2456 |
| add_stake | spend | `Constr(0, [])` | none | 15764 |
| withdraw_stake | spend | `Constr(1, [])` | none | 1307 |
| withdraw_stake | mint | `Constr(1, [Bytes])` | owner_pkh | 1303 |

## Notes

- **Owner NFT stays in the script (2026-04-13 fix):** The on-chain contract requires the owner NFT to be locked in the script UTxO, not held in the staker's wallet. This was confirmed by inspecting the Aiken source (`validators/staking.ak`) and verified against all 3 reference transactions. The `main.tx3` was corrected: `stake` now sends the owner NFT to the script output, and `add_stake`/`withdraw_stake` no longer require the owner NFT in the staker's `source` input.
- The `add_stake` transaction required a workaround: the datum spread syntax (`...current_stake`) caused a TRP error (`property index 0 not found in None`). The fix was to replace it with explicit datum fields and pass `staked_at_time` as an additional parameter.
- The `main.tx3` protocol matches the active contract (PlutusV3, script hash `497a8b...`) as published on the Strike Finance GitHub repository.
- All environment values (policy IDs, reference script UTxO, script address) are loaded from `.env.mainnet` via the built-in mainnet profile.

## How to Reproduce

```bash
cd protocols_tx3/strike-staking

# stake
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/stake.json
# select: stake

# add_stake
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/add_stake.json
# select: add_stake

# withdraw_stake
trix invoke --profile mainnet --skip-submit --args-json-path invoke-args/withdraw_stake.json
# select: withdraw_stake
```
