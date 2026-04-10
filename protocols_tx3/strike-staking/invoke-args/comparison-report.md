# Strike-Staking: CBOR Comparison Report

Generated 2026-03-30. Compares transactions built by `trix invoke --profile mainnet --skip-submit` against real on-chain transactions from the active Strike Finance staking contract (`497a8b0085517f1c9065cf3006af4c266454b39c6fa32a9d116c75ee`).

## Reference Transactions (real on-chain)

| Operation | Tx Hash | Block |
|-----------|---------|-------|
| stake | `f70239fa91bcb0df496c011465e440e8cd97955231df956c7de3820e1c861a80` | 13140934 |
| add_stake | `939737ecd4f1ed2cab613a63606802287e79b0525767ddaff6057bbafd28bfab` | 13140518 |
| withdraw_stake | `60f83cf13c421d33a039cd902b57dd67fec4f4afbcdbade5a597b3f816d58b06` | 13140491 |

## Generated Transactions

| Operation | Tx Hash (unsigned) | Args File |
|-----------|--------------------|-----------|
| stake | `0139933da5ca48a317c8754aa8b2729fb360ecb924ed4a5e0c956561181d4954` | `stake.json` |
| add_stake | `1b20ed39974d889164c7eec8b578dd19ffac3b171ccb4048b7fb2b3d8fe82775` | `add_stake.json` |
| withdraw_stake | `09c99128bd9fbd87e4f6b58e556825af0d39658bd0141300e440981ad66f6339` | `withdraw_stake.json` |

## Test Wallet

All invocations used a real mainnet wallet that holds the required assets:

- **Address:** `addr1qyzvvlj4d4rte6fkal0jmthpmq4qxayynfm2u5qkxxw7kauhrak4m7z3wtkunk0yvcrx0f4w0zdtnsmek7576lggsewqwzc6f7`
- **Payment credential:** `04c67e556d46bce936efdf2daee1d82a0374849a76ae5016319deb77`
- **Assets:** Owner NFT + tracker token + ~500M STRIKE + ~62 ADA
- **Staking UTxO (at script):** `fc86409d1d99323e31dcff833ade98722c4ba8ef5a52e465b738114dfc1a4a74#0` (9.5B STRIKE staked)

## Structural Comparison

### stake

| Field | Real | Ours | Match |
|-------|------|------|-------|
| Body fields (structural) | inputs, outputs, fee, ttl, mint, script_data_hash, collateral, required_signers, collateral_return, total_collateral | inputs, outputs, fee, ttl, mint, script_data_hash, collateral, required_signers, network_id, reference_inputs | see notes |
| Output count | 2 (script + change) | 2 (script + change) | YES |
| Output[0] destination | staking script address | staking script address | YES |
| Output[0] tokens | tracker(1) + STRIKE(staked_amount) | tracker(1) + STRIKE(staked_amount) | YES |
| Output[0] datum | StakingDatum (3 fields) | StakingDatum (3 fields) | YES |
| Output[1] destination | staker wallet | staker wallet | YES |
| Output[1] tokens | owner NFT(1) + change STRIKE | owner NFT(1) + change STRIKE | YES |
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
