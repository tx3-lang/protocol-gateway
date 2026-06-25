# Indigo Protocol — TX3 V3 Migration & Comparison Report

**Date:** 2026-06-24. Re-pointed from the dead VX deployment to the on-chain **V3**
deployment (Indigo migrated VX→V3 on 2026-05-28). Toolchain: trix 0.26.2 / tx3-lang 0.23.0.
On-chain investigation + verdicts in `investigation/v3-findings.md`.

Verification method (per request): `trix check`/`build` + `trix inspect tir` for datum/redeemer
shapes, **plus headless live-resolve** against the mainnet TRP
(`cardano-mainnet.trp-m1.demeter.run`, via direct `trp.resolve` JSON-RPC — the local cshell
wallet is empty, so `trix invoke` can't run), comparing the resolved CBOR datums against the
real V3 on-chain datums.

## Status summary

| # | Transaction | V3 status | Verification |
|---|---|---|---|
| 1 | `create_staking`   | ✅ working | **live-resolved** — datums match on-chain |
| 2 | `adjust_staking`   | ✅ working (empty lock) | **live-resolved**; populated vote-locks blocked (tuple lowering) |
| 3 | `unstake`          | ✅ working | **live-resolved** — manager co-spend + NFT burn |
| 4 | `adjust_cdp_mint`  | ✅ working (deposit) | **live-resolved** — 5-field CDP datum + interest collector |
| 5 | `adjust_cdp_burn`  | ✅ working (repay) | **live-resolved** — repay + interest settle |
| 6 | `close_cdp`        | ✅ working | **live-resolved** — CloseCdp(1); matches on-chain `37783f745c0d…` |
| 7 | `withdraw_cdp`     | ✅ working | **live-resolved** — 2 Pyth withdrawals; body[5]+redeemers match on-chain |
| 8 | `borrow_cdp`       | ✅ working | **live-resolved** — 2 Pyth withdrawals + Treasury `C4` + OutRef datum |
| 9 | `create_sp_account`| ⛔ blocked (tx3 u64) | TIR shape ✓; values unencodable (see below) |
| 10 | `adjust_sp_account`| ⛔ blocked (tx3 u64) | TIR shape ✓; values unencodable |
| 11 | `close_sp_account` | ⛔ blocked (tx3 u64) | TIR shape ✓; values unencodable |
| – | `create_cdp`       | ⛔ blocked (input index coupling) | see below |

**Deployed `protocols/indigo.tii` exposes 11 methods** (8 working: 3 staking + 5 CDP
[deposit/repay/close/withdraw/borrow]; 3 SP blocked-and-documented with a `[BLOCKED — tx3 …]` description
prefix). `create_cdp` is absent.

**CDP scope (validated vs `@indigo-labs/indigo-sdk` 0.3.28 + on-chain).** The 5 CDP txs cover the full write
surface except create:
- `adjust_cdp_mint` = **deposit collateral** (debtAdjustment 0, `OracleVoid`, no treasury).
- `adjust_cdp_burn` = **repay debt** (`OracleVoid`, no treasury).
- `close_cdp` = **normal active close** (`CloseCdp` redeemer).
- `withdraw_cdp` = **remove collateral** — runs the Pyth price check via **two 0-amount `cardano::withdrawal`
  blocks** (the Pyth-state-update validator `4949403eec` with redeemer `[pythMessage]`, and the per-iAsset feed
  validator `1d4c0f85` with redeemer `{derivedPrice, unit}`). No treasury. The 4th AdjustCdp field
  `priceOracleIdx` = `OracleVoid` (Pyth-priced). Resolved tx `body[5]` carries both withdrawals; redeemers
  byte-match the on-chain price check.
- `borrow_cdp` = **mint more debt** — withdraw machinery + the **Treasury** (debt-minting fee via a `Constr(4)`
  redeemer + an OutRef-based continuity datum `C0[C0[OutRef(treasuryIn)], OutRef(cdpIn)]`). Total minted =
  debtAdjustment + interest; treasury_fee → Treasury, interest → Collector, (debt − fee) → user. The OutRef
  datum is built from caller-passed txHash+index (tx3 can't decompose a UtxoRef into a datum).

The Pyth-withdrawal mechanism was first proven by a standalone probe (a `cardano::withdrawal` against the real
Pyth reward account resolves; `body[5]` + reward-purpose redeemer + `script_data_hash` all correct).

**`create_cdp` — blocked by INPUT-index coupling (not the Pyth/treasury machinery).** Its 11-field CDP-Creator
redeemer carries `creatorInputIdx` = the cdpCreator UTxO's position among ALL inputs (sorted by txid).
Borrow/withdraw avoid this because the treasury redeemer references the CDP by **OutRef value** (computable
from params); create needs the cdpCreator's **input index**, which depends on the resolver's coin selection
(which user UTxOs get pulled, and their txids) — not predictable by the caller before resolve. So create stays
out of reach (the output/ref indices are predictable; the input index is the blocker).

## Detailed generated-vs-on-chain comparison (2026-06-25)

Generated each tx (cshell `trix invoke --skip-submit` and the equivalent headless `trp.resolve`),
fetched the real on-chain reference (`tx_cbor`), decoded both with one decoder, compared **by value**
(definite vs indefinite CBOR arrays are value-equivalent). Result: **all 8 working txs are structurally
identical to their on-chain reference.**

| tx | verdict |
|---|---|
| create_staking | ✅ scripts + mint (+1 STAKING_POSITION) + datums (Manager `C0[C0[total,C0[snap]]]`, Position `C1[C0[owner,[],C0[snap]]]`) match; snapshot value identical |
| adjust_staking | ✅ co-spend redeemers `C3[delta]`(AdjustStakedAmount) + `C1[]`(UpdateTotalStake); datums match |
| unstake | ✅ burn −1 STAKING_POSITION; redeemers `C4[]`(Unstake) + `C1[]`; datums match |
| adjust_cdp_mint | ✅ ref scripts EXACT; AdjustCdp `C0[ts,0,Δcol,OracleVoid]`; 5-field CDP datum + collector `C0[]` match |
| adjust_cdp_burn | ✅ ref scripts EXACT; AdjustCdp `C0[ts,−Δdebt,0,OracleVoid]`; datums match |
| close_cdp | ✅ `CloseCdp C1[ts]`; burns NFT + full debt; collector `C0[]`. (per-iAsset refs differ only because the on-chain ref closed an iBTC CDP vs my iUSD) |
| withdraw_cdp | ✅ **two Pyth withdrawals** (`f1·4949403eec` state + `f1·1d4c0f85` feed) + redeemers (feed price `C0[C0[a,b],C0[]]` + Pyth message) match; no treasury (correct) |
| borrow_cdp | ✅ two Pyth withdrawals + **Treasury `C4`** redeemer + **OutRef continuity datum** `C0[C0[C0[OutRef],_],C0[OutRef]]` — all match the on-chain borrow `6b8e6d44` |

**Metadata:** NONE — no Indigo V3 tx (generated or on-chain) carries transaction metadata (no aux-data hash).

**Bug caught by the comparison + FIXED:** `withdraw_cdp`/`borrow_cdp` were missing the **iAsset config
reference input** (`fb6cd010#0`) that `adjustCdp`'s ref set requires (the on-chain borrow references
`fb6cd010` twice = config #0 + state #2; mine had only #2). Added `iasset_config_utxo` to both. After the fix,
the only remaining ref-set difference is the **interest-oracle instance** (a singleton updated over time —
mine uses the current unspent UTxO, the reference tx used the one current at its block).

**Legitimate (expected) differences in every comparison:** input UTxOs, fees, validity slots, the specific
amounts/owner/CDP/collateral-asset, input & output ORDER (Cardano sorts inputs/refs by txid), and
definite-vs-indefinite CBOR array encoding.

## Reference transactions (real V3 on-chain, used as ground truth)

| Flow | tx hash |
|---|---|
| create_staking (mint +1 STAKING_POSITION) | `e7e058e762cc72c24fc6729e0080e624ad9ad761b8306fe37e05b2b6802aa72d` |
| unstake (burn -1) | `1a16fe014ba8c35fd0e8058b3d534699e0af25546f3e0df22a9fca40747cd173` |
| adjust_staking | `21cdb9dc99e537916ffd1163ba08b9707302605a31fec76a37eeee6beebc1b70` |
| adjust_cdp_mint (add collateral + interest) | `813eef7a84bf1d0538068c722f62fffecabc7a74cfbeb9f418053120f0f92ab4` |
| adjust_cdp_burn (repay + interest) | `564ff734c806f484e5ac93a16df3caa5505a1f817ccc6e17911e95ac7b92121e` |
| create_cdp (reference, NOT implemented) | `bb864aefa79f8fca2cafb690b6d37be25a51402cc06347ee2ffec0491f823350` |
| SP create-request | `85b683edf94e2169ff28226e75ce9d00cb2b32cb3c97b18f1ba705a183f58720` |
| SP adjust-request | `e135782e31e8d2daed763e1ec138ae7f29e2b41322549268067de41bb6006db0` |

## Datum / redeemer comparison (working txs)

### Staking (live-resolved, exact match)
- Manager `C0[C0[totalStake, C0[snapshotAda]]]`; Position `C1[C0[owner, lockedAmount, C0[snapshotAda]]]`.
- Empty `lockedAmount` resolves to `[]` = CBOR `80` (byte-match on-chain; the old `{}` map `a0` was wrong).
- `create_staking`: manager redeemer `C0[pkh]` (StCreateStakingPosition), `totalStake += indy` via inline `Add`, mints +1.
- `adjust_staking`: co-spends manager (`C1[]` UpdateTotalStake) + position (`C3[delta]` AdjustStakedAmount); `totalStake ± delta`.
- `unstake`: co-spends manager (`C1[]`) + position (`C4[]` StUnstake); **burns** -1 STAKING_POSITION; `totalStake -= amount`.

### CDP adjust (live-resolved, exact match)
- CDP datum (5 fields, V3): `C0[C0[ JustPkh(owner), iasset, C0["",""], minted, ActiveTracking(ts, accumulator) ]]`.
  Byte-equivalent to on-chain (e.g. resolved `…1000100, C0[1782322416000, 424025302629185631]…`).
- Redeemer `AdjustCDP` = `C0[ts, minted_change, collateral_change, C2[](CKAda)]`.
- Co-spends the Interest Collection validator (`2cac220a78`, redeemer `Collect` = `C0[]`); the accrued
  interest is minted and added to the collector output (unit datum `C0[]`).
- 3 read-only DATA reference inputs (oracle price, iAsset config, iAsset state).
- Accumulator (~4.2e17) is < u64 → encodes cleanly.

## Blockers (documented)

### Stability Pool — tx3 u64 integer ceiling (HARD)
`IntoData for i128` in tx3-cardano does `Int::try_from(i128).unwrap()`; pallas `Int` is **u64-bounded**,
so tx3 **panics on any integer > u64::MAX (18446744073709551615)** — it never emits CBOR bignums.
Confirmed by binary-search against the live TRP: u64::MAX resolves, u64::MAX+1 → HTTP 502 (panic).
The SP account/pool snapshot carries 18-decimal fixed-point accumulators that routinely exceed u64
(real on-chain create `'d'` = `381e18` ≈ 3.8e20; adjust accounts reach `6.5e25`). So the SP datum
SHAPES are correct & TIR-verified, but the VALUES can't be built for any realistic deposit
(only ≤ 18 raw units fit). The VX impl never hit this — its invoke-args used all-zero snapshots.
Big ints up to u64 can be passed as JSON **strings** (`value_to_bigint` parses decimal/hex → i128),
but the lowering still caps at u64.

### Staking populated `lockedAmount` — tuple lowering (PARTIAL)
`lockedAmount` is an association list of `(PollId, LockedEntry)` where the on-chain entry is a
**2-array** `[pollId, Constr0[vote, end]]` (NOT a Constr, NOT a map). Modeled as
`List<Tuple<Int, LockedEntry>>`. The **empty** case resolves (`{"list":[]}` → `80`). A **populated**
entry fails at resolve: `error coercing Tuple(...) into PlutusData` — tx3's `TryIntoData for Expression`
has no `Tuple` arm (it lowers Struct/List/Map but not Tuple). So positions with active governance
vote-locks (rare) can't be adjusted; the common empty-lock case works, and `unstake` is unaffected
(it doesn't reconstruct the lock).

### CDP create / close — V3 redesign beyond tx3 (NOT IMPLEMENTED)
- `create_cdp`: consumes the **Treasury** (collects the mint fee, OutRef-based datums), runs **two
  withdrawal-purpose "reward" validators** — one carrying a **~150-byte Pyth price proof** — and uses an
  **11-field CDP-Creator redeemer whose trailing values are OUTPUT INDICES** that must match the tx's
  exact output ordering. tx3 has `cardano::withdrawal` blocks, but cannot guarantee the index coupling,
  and the Pyth proof / treasury continuity reduce most of the tx to opaque caller blobs.
- `close_cdp`: several on-chain variants (normal active close, **frozen** close, **liquidation**); the
  frozen/liquidation path co-spends the Stability Pool (whose >u64 snapshot is unencodable) and the
  Treasury. A faithful, resolving implementation is a separate, larger effort.

## How to reproduce

```bash
cd protocols_tx3/indigo
trix check && trix build
# headless live-resolve (helper replicates tx3-sdk into_resolve_request against the mainnet TRP):
python3 <scratchpad>/resolve.py create_staking   invoke-args/create_staking.json
python3 <scratchpad>/resolve.py unstake          invoke-args/unstake.json
python3 <scratchpad>/resolve.py adjust_staking    invoke-args/adjust_staking.json
python3 <scratchpad>/resolve.py adjust_cdp_mint   invoke-args/adjust_cdp_mint.json
python3 <scratchpad>/resolve.py adjust_cdp_burn   invoke-args/adjust_cdp_burn.json
```

CBOR encoding note: tx3 emits **definite-length** arrays/maps; Aiken/Plutus contracts often emit
**indefinite-length** (`9f…ff`). These are value-equivalent but not byte-identical; compare by decoded
value (or recompute the hash in both conventions), as established for snek-fun/vyfi.
