# Indigo V3 — On-chain investigation findings (2026-06-24)

Re-targeting the tx3 protocol from the dead VX deployment to V3 (migrated 2026-05-28).
Investigated live via Koios. Decoder + saved tx JSON in session scratchpad.

## 1. Env re-point (deterministic, fully verified)

All V3 validator refs are **unspent** and their `reference_script.hash` matches `validatorHashes`
(Koios `utxo_info`). Enterprise addresses re-derived with bech32 (self-check reproduces the old
VX CDP-spend addr). Mapping VX → V3:

| .env var | V3 value |
|---|---|
| CDPSCRIPT (enterprise, ff0b10bf) | `addr1w8lsky9l7g8yk695j9yjukaxeqzg5uz8vwc2gh8zn9w6p0sy523pp` |
| CDPCREATORSCRIPT (95c1458d) | `addr1wx2uz3vd4kwlmhatytgk8lxthx5dujp6ec0pexgtra0ralc6sxw67` |
| COLLECTORSCRIPT (68f23cd9) | `addr1w950y0xehm7z5a9qqcstf8tx545u6yct9yxdwql9cyvfrlg4c5nk6` |
| STABILITYPOOLSCRIPT (1c53ed6f) | enterprise `addr1wyw98mt0v9ng0v6q4jpswtkxt2tcwkpuq8tt4cp3fcwkr5qpqnput` BUT real SP/account UTxOs sit at **addr1x** (script pay + script stake `b8358aad`): `addr1xyw98mt0v9ng0v6q4jpswtkxt2tcwkpuq8tt4cp3fcwkr59cxk92m5cvvr46z6rq3t27sa2e96dhewx8qzp8eh58lx3s9jz2y6` |
| STAKINGSCRIPT (112996eb) | `addr1wygjn9htqy0jp6kv9r0dsmx7zaa98mfzye8qzvgenjvr5ls2dl3wc` |
| CDP_SPEND_REF | `f1275d84896bff1e557b7dc7d9cd4030fcd5539ae5ec99c3dd97d818c75c950a#0` |
| CDP_CREATOR_REF | `528a870ae53cd77dde8ace624ffd25610057e059466ded1d5d525d60141bf667#0` |
| COLLECTOR_REF | `4dfcc06496c2aee494909d7508a959d29465061db4cefb26ce331c971a9f606d#0` |
| STABILITY_POOL_REF | `8b76f66191bc65f0a7e4106b85a95b5524329a00049932f75faebdce4651d249#0` |
| STAKING_REF | `03a6b59ef3ca62527282aaf3461f374009392eb8a79aad635196077c17e74b28#0` |
| IASSET_MINT_REF | **`8fac023fd9556f66ffdac8cb71879fabd6995480f7ad976213d1e1ee85de75cd#0`** (iAssetTokenPolicyRef — carries the f66d78b4 mint policy; the handoff's guess `0c55ef7c` is the IASSET *config* auth token `97da12de`, NOT the mint policy) |
| CDP_NFT_MINT_REF | `c0a4c2ad340da8686c723a21b0a029aefee650fcaf5ef964742f499efb7c21f8#0` — SAME as VX |
| STAKING_POSITION_MINT_REF | `71dc6b81e8832192bb28ecbc6a4f71b6e0dc0407c708f169020804371450b4e7#0` — SAME as VX |

**Minting policies / token identities UNCHANGED** (asset identity preserved): iAsset `f66d78b4`,
CDP NFT `708f5e6d`/CDP, CDP_CREATOR `735b3714`/CDP_CREATOR (**confirmed unchanged in cdpCreatorParams** —
resolves NEXT-STEPS §2c ⚠), INDY `533bb94a`, staking position `fd0d72fa`, staking-mgr NFT `24b45841`,
iAsset-config NFT `97da12de`, SP NFT `3f28fb7d`.

**New V3 tokens (multi-collateral + bookkeeping):** collateralAssetAuthToken `b7d412f5`/COLLATERAL_ASSET,
versionRecordToken `d626ddf3`/VERSION_RECORD, upgradeToken `ca72f111`/UPGRADE, SP_EPOCH `98ebc5df`,
SP_ACCOUNT `443c51db`. Pyth State `c935c937`.

## 2. CDP flows (create/adjust/close) — MAJOR redesign, at/beyond tx3's edge

Evidence: create_cdp `bb864aef…` (iJPY), `cae4fdde…` (iUSD), `f264dcef…` (iUSD, also processes a ROB order).

- **CDP datum: 5 inner fields** (was 4). New `collateral AssetClass` `C0[policy,name]` at index 2
  (`("","")` = ADA). Shape:
  `C0[C0[ Maybe(pkh), iasset, C0[colPolicy,colName], minted, C0[ts_ms, accumulator] ]]`.
- **CDP-Creator redeemer: 11 fields** (was 4): `C0[pkh, mint, collateral_lov, ts_ms, i, i, i, i, i, i, C2[]]`
  — the 6 trailing values are **output/input indices** that must match the tx's exact output ordering
  (e.g. `…,2,1,0,0,8,6,C2[]`), plus a trailing Constr enum.
- **Treasury** validator (`46ffe1d1`) is consumed and returned — collects the iAsset mint **fee** (e.g.
  5_000_000 iJPY of a 5_000_000_000 mint). Treasury in/out datums are OutRef-based (`C0[C0[C0[OutRef]],C0[OutRef]]`).
- **cdpCreator output** carries an OutRef datum now (`C0[C0[OutRef]]`), not a unit datum.
- **2+ reward/withdrawal-purpose script executions**: `0db1238594` (iasset reward, redeemer `C0[C0[Int,Int],C0[]]`)
  and `4949403eec` (redeemer is a **~150-byte blob = the Pyth price proof**). tx3 *can* express these via
  `cardano::withdrawal { from, amount: 0, redeemer }`, but the redeemer is opaque caller data.
- **Data reference inputs**: per-iAsset config (`fb6cd010`/`15d8ac12` = `C0[C0[iasset, denom, ratios…, collateral-iasset]]`),
  iAsset state (`f539db5f` = `C1[…]` with collateral AssetClass + oracle script ref), and an **oracle price**
  UTxO still in the old `C0[nonce, C0[price], expiration]` shape (`ae4e00aa`).
- Adjust/close CDP redeemers (`AdjustCDP`/`CloseCDP`) and their treasury/reward involvement not yet
  fully re-derived — expected to be comparably complex.

Feasible in tx3 *in principle* (withdrawal blocks exist; everything else becomes caller params), but the
result is a large, index-coupled multi-script tx with many opaque caller-provided blobs (Pyth proof,
accumulator, every output index, treasury datums). High effort, high fragility. These txs are normally
built by Indigo's own frontend SDK.

## 3. Staking flows — tractable rewrites

Datums now **populated** (V3 activated the structured staking datum), and they **match the current
main.tx3 model**:
- Manager `C0[C0[totalStake, C0[snapshotAda]]]`; Position `C1[C0[owner, lockedAmount, C0[snapshotAda]]]`.
- `lockedAmount` is `Map<Int, C0[Int,Int]>` (confirmed populated `{124 → C0[10000000000, 1780432450000]}`).
- Redeemer indices confirmed: 0 StCreateStakingPosition{pkh}, 1 StUpdateTotalStake, 3 StAdjustStakedAmount{delta}, 4 StUnstake.

Deltas vs current main.tx3:
- **create_staking: already CORRECT** (spends Manager idx0, mints +1, manager totalStake+=indy, position datum). Just re-point env.
- **adjust_staking: WRONG** — V3 co-spends the **Manager** (`StUpdateTotalStake` idx1) + position (`StAdjustStakedAmount{delta}` idx3); both datums structured (not Unit); manager totalStake changes by delta. Current model spends only the position with UnitDatum. Rewrite.
- **unstake: WRONG** — V3 co-spends the **Manager** (idx1) + position (`StUnstake` idx4), **burns −1 STAKING_POSITION**, position input datum is structured; user gets INDY back (NFT burned). Current model spends only position, no manager, no burn. Rewrite.

Evidence: create `e7e058e7…`,`e815b4a3…`; unstake `1a16fe01…`,`bd2d34a4…`; adjust `21cdb9dc…`,`c624c13a…`,`ead36421…`.

## 4. Stability Pool flows — significant datum/redeemer reshape, but tractable

SP/account UTxOs at **addr1x** (script+script stake). NO Collector, NO Treasury, NO reward validators
in the account lifecycle.
- **Snapshot reshaped**: VX `C0[p,d,s,epoch,scale]` (5 ints) → V3 `C0[C0[Int], C0[Int], Int, Int]` (4 fields).
- **PoolContent**: 3 fields now — `C0[iasset, snapshot, rewards_map]` (NEW rewards map).
- **AccountContent**: 6 fields now — `C0[owner, iasset, snapshot, deposits_map, request, created_at]`
  (NEW deposits_map at idx3 + created_at POSIXTime at idx5).
- **AccountAction**: ActCreate `C0[]`; ActAdjust `C1[amount, addr]` where **addr is a structured Address**
  (not Bytes); ActClose `C2[addr, lovelace]` (**2 fields** now, structured addr + lovelace).
- **StabilityPoolRedeemer ctor meanings changed**: 0=RequestAction{action} (OK); 1=ProcessRequest pool-side
  `[i,j]`; 2=ProcessRequest account-side `[i,j,list,ts]`; 4=LiquidateCDP `[i]`. VX's enum past ctor 0 is wrong.
- **create_sp_account is a plain build tx** (no script spend, no mint) — fund a new account output at the SP
  addr with `Just(Create)` + 5 ADA fee; the SP_ACCOUNT token is minted later by the batcher's process step.
- adjust/close request txs spend the account with `RequestAction(Adjust|Close)` + only the SP validator ref.
  **Drop the Collector** input the VX model used.

Evidence: create-req `85b683ed…`; adjust-req `e135782e…` (deposit), `83424896…` (withdraw); close-req `3d19c13f…`.

## 5. Net assessment

- Env re-point: done/verified.
- Staking ×3 + SP ×3 = 6 user-facing txs: tractable V3 rewrites, fully scoped above.
- CDP ×4 (create/adjust_mint/adjust_burn/close): major redesign, at the edge of tx3, many opaque caller
  params; high effort/risk. These are the protocol's core product.

## 6. Implementation results (2026-06-24)

**Done + live-resolved (mainnet TRP, datums byte-/value-matched on-chain):**
- `create_staking`, `unstake`, `adjust_staking` (empty lock) — staking datums match; lockedAmount empty
  list → CBOR `80`; manager co-spend + NFT burn confirmed; inline `Add`/`Sub` on totalStake.
- `adjust_cdp_mint` (deposit collateral), `adjust_cdp_burn` (repay debt), `close_cdp` (normal close) — 5-field
  V3 CDP datum (collateral AssetClass `C0["",""]`) + Interest Collection co-spend (`2cac220a78`, `Collect`);
  oracle/config/state as DATA refs; accumulator (~4.2e17, < u64) encodes cleanly. NEW env:
  `INTERESTCOLLECTORSCRIPT`, `INTEREST_COLLECTION_REF`. `close_cdp` confirmed vs on-chain `37783f745c0d…`
  (`CloseCdp(1)`, burns debt + NFT, no treasury/price-oracle/SP). All three are the **safe** CDP ops.
- `withdraw_cdp` (remove collateral) + `borrow_cdp` (mint more debt) — **DONE + live-resolved.** Both run the
  Pyth price check via **two 0-amount `cardano::withdrawal` blocks** (Pyth-state-update validator `4949403eec`
  redeemer `[pythMessage]`; per-iAsset feed validator `1d4c0f85` redeemer `{derivedPrice, unit}`). `borrow_cdp`
  adds the **Treasury** (debt-minting fee via a `Constr(4)` redeemer + an OutRef continuity datum
  `C0[C0[OutRef(treasuryIn)], OutRef(cdpIn)]`, built from caller-passed txHash+idx). Resolved tx `body[5]` has
  both withdrawals; redeemers byte-match the on-chain borrow `6b8e6d44…`. NEW env: `PYTHSTATEWITHDRAW`,
  `PYTH_STATE_WITHDRAW_REF`, `TREASURYSCRIPT`, `TREASURY_REF`; NEW per-call party `FeedValidator`.
  **The Pyth-withdrawal mechanism was first proven by a standalone probe** (`cardano::withdrawal` against the
  real Pyth reward account resolves; `body[5]` + reward-purpose redeemer + `script_data_hash` all correct).
- **`create_cdp` — blocked by INPUT-index coupling** (NOT the Pyth/treasury machinery, which now works). The
  11-field CDP-Creator redeemer carries `creatorInputIdx` = the cdpCreator UTxO's position among ALL inputs
  (Cardano sorts inputs by txid). Borrow/withdraw avoid this because the treasury redeemer references the CDP
  by **OutRef value** (computable from params); create needs the cdpCreator's **input index**, which depends on
  the resolver's coin selection (which user UTxOs get pulled + their txids) — not predictable by the caller
  before resolve. Output/ref indices are predictable; the input index is the blocker.
- **`cardano::withdrawal` feasibility probe — PASSED (2026-06-24).** A standalone tx3 tx with
  `cardano::withdrawal { from: <script reward acct>, amount: 0, redeemer: <blob> }` + a `reference` block for
  the validator's ref script **resolves on the mainnet TRP** against the real Pyth reward validator
  (`4949403eec…`). Decoded the resolved CBOR: the withdrawal lands in `body[5]` keyed by the script-stake
  account (`f1`+hash) → 0; the validator ref script is attached in `body[18]` (tx3 auto-matched by hash); the
  witness redeemers carry `(purpose=3 reward, idx 0)` with the caller redeemer + ex-units; `script_data_hash`
  present. **So the Pyth-withdrawal mechanism is viable in tx3** — borrow/withdraw/create are NOT blocked by
  the withdrawal. Remaining work for borrow: 2 withdrawal blocks (proven) + the Treasury continuity (a normal
  script spend with a `C4` redeemer + OutRef datum) + the Pyth proof as a `List<Bytes>` param. Withdraw is
  simpler (Pyth withdrawal for the price check, no treasury). create still adds the 11-field index coupling.

**Blocked / not implemented:**
- **Stability Pool ×3 — tx3 u64 ceiling (HARD).** `IntoData for i128` panics on integers > u64::MAX
  (pallas `Int` is u64-bounded; tx3 never emits CBOR bignums). Verified via the live TRP: u64::MAX
  resolves, u64::MAX+1 → 502. SP snapshot decimals (`381e18`+) exceed u64 → datum VALUES unencodable.
  Datum SHAPES are TIR-correct. Kept in the deployed `.tii`, prominently documented as blocked.
- **Staking populated lockedAmount — tuple lowering (PARTIAL).** tx3 `TryIntoData for Expression` has no
  `Tuple` arm; a populated `[pollId, Constr0[vote,end]]` entry fails resolve (`error coercing Tuple…`).
  Empty case works; `unstake` unaffected. Only positions with active governance vote-locks (rare) affected.
- **CDP create / close — not implemented.** create_cdp needs Treasury continuity + two withdrawal-purpose
  reward validators (one with a ~150B Pyth proof) + an 11-field index-coupled creator redeemer; close_cdp
  has normal/frozen/liquidation variants (frozen co-spends the unencodable SP + Treasury). Faithful
  resolving impls are a separate, larger effort. Decoded reference: create `bb864aef…`, frozen-close
  `afbe8d0f…`. The VX CDP scaffolding is preserved in git history.

**Deployed `protocols/indigo.tii`: 8 methods** — 5 working (3 staking + 2 CDP adjust), 3 SP blocked-documented.

## 7b. Cross-check vs the official v3 SDK (`@indigo-labs/indigo-sdk` 0.3.28, 2026-06-24)

Fetched the published package with `npm pack @indigo-labs/indigo-sdk@0.3.28` (public; full TS source).
NOTE: the SDK repo checkout shared was **v0.2.43 = the legacy PRE-v3 SDK** (4-field CDP datum, 4-field
CreateCDP); the v3 shapes live only in the published 0.3.x line (`cdp/types-new.ts`). The MCP targets the
same live v3 (`mainnet-system-params-v3.json`) and depends on `indigo-sdk ^0.3.28`.

**Confirmed (my on-chain-derived impl matches the SDK exactly):**
- `CDPContent = {cdpOwner, iasset, collateralAsset, mintedAmt, cdpFees}` — 5 fields, `collateralAsset`
  (AssetClass) at index 2. ✓
- `cdpFees.ActiveCDPInterestTracking{lastSettled, unitaryInterestSnapshot}` = my `ActiveTracking`. ✓
- CDP redeemer order `AdjustCdp=0, CloseCdp=1, RedeemCdp=2, FreezeCdp=3, MergeCdps=4, MergeAuxiliary=5,
  Liquidate=6, …, UpgradeVersion=8`. ✓ So the on-chain `Constr(6)` "frozen close" is a **Liquidate**; a
  normal close is `CloseCdp(1)`.
- adjust co-spends the **interest collection** validator + reads oracle/iAsset-config/iAsset-state DATA
  refs; `mintedAmountChange = debtAdjustment + interestAmt`. ✓

**Corrected:** the AdjustCdp 4th field is **`priceOracleIdx: OracleIdx`** (`OracleRefInputIdx{idx}`=Constr0 |
`OracleOutputIdx{idx}`=Constr1 | `OracleVoid`=Constr2), NOT a "collateral kind." SDK rule:
`isMintOrWithdraw = debtAdjustment > 0 || collateralAdjustment < 0` ? `OracleRefInputIdx{ref idx}` :
`OracleVoid`. So my hardcoded `C2[]` is right for **add-collateral / repay** but a real **borrow / collateral
withdraw** needs `OracleRefInputIdx{idx}` (idx = oracle position in tx3's reference_inputs — index-coupled).
Fixed: replaced the mis-named `CollateralKind` type with `OracleIdx` and made the 4th field a caller param
(`price_oracle_idx`); re-resolved both adjust txs (OracleVoid case byte-identical). For borrow/withdraw the
caller passes `{"struct":{"constructor":0,"fields":[{"int":<idx>}]}}`.

**create_cdp confirmed hard:** `CreateCDP` redeemer = `{cdpOwner, minted, collateralAmt, currentTime,
creatorInputIdx, creatorOutputIdx, cdpOutputIdx, iassetRefInputIdx, collateralAssetRefInputIdx,
interestOracleRefInputIdx, priceOracleIdx}` — **6 explicit input/output/ref indices** that must match tx3's
exact ordering. Still beyond tx3's deterministic control.

## 7. Reusable tx3 limitations discovered (file as quirks)

1. **u64 integer ceiling**: tx3 panics (`Int::try_from(i128).unwrap()`) on any datum/redeemer integer
   > u64::MAX; it never emits CBOR bignums. Blocks any Plutus datum with large 18-decimal fixed-point
   values (Indigo SP, many DeFi). No workaround.
2. **Tuple → PlutusData not lowered**: `Tuple<…>` compiles & appears in TIR but the cardano coercion has
   no `Tuple` arm → resolve-time `error coercing Tuple into PlutusData`. Empty `List<Tuple<…>>` is fine
   (no elements to coerce); populated fails. Use records (Constr) where the on-chain shape is a Constr;
   tuples only work if never actually populated.
3. **List param wire form**: a top-level `List<T>` param must be passed **tagged** `{"list":[...]}`, not a
   bare `[]` (bare hits `coerce_bare(List)` → `target type not supported: List`). (Contradicts an earlier
   fluid-aquarium note that bare arrays work — re-verified bare fails on this resolver, 2026-06-24.)
4. **Big-int args as strings**: integers up to u64::MAX can be passed as JSON **strings**
   (`value_to_bigint` parses decimal/hex → i128); above u64::MAX the lowering still panics (see #1).
</content>
</invoke>
