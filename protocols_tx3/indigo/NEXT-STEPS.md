# Indigo — V3 Migration: Next Steps

> **Status (2026-06-16):** Indigo redeployed its contracts on-chain (**"V3"**) and migrated to
> them on **2026-05-28**. This tx3 protocol (`main.tx3` / `.tii` / `.env.mainnet`) still targets
> the **old "VX" deployment, which is now dead**. The protocol must be re-pointed to V3 and
> re-verified before it builds valid transactions again.
>
> This file is a self-contained handoff so a fresh agent can resume without re-investigating.
> Companion research (pre-migration, still useful for structure/redeemers): `investigation/indigo-research.md`,
> `investigation/tx3-limitations-indigo.md`, `invoke-args/comparison-report.md`.

---

## 1. The finding (what happened)

- Indigo published a new config: `https://config.indigoprotocol.io/mainnet/mainnet-system-params-v3.json`
  (there is also a `-v2.json`; the no-suffix `mainnet-system-params.json` returns HTTP 403).
- The V3 reference scripts were all created on **2026-05-27**.
- The protocol cut over from VX → V3 on **2026-05-28**.
- Our `.env.mainnet` (and the baked-in `.tii`) describe the VX deployment, which our April 2026
  research validated as ~100% structural match — but that was **before** this migration.

### Evidence (verified on-chain via Koios — re-runnable, see §6)

| Address (consumed in every CDP op) | Last activity |
|---|---|
| **CDP_CREATOR — old/VX** (`addr1wyy3pau5vxn37arc9hx52rezkrpv4sc6kqmtvmyjry64mxgefqrn0`) | **2026-05-28 15:09 UTC — then dead** |
| **CDP_CREATOR — v3** (`addr1wx2uz3vd4kwlmhatytgk8lxthx5dujp6ec0pexgtra0ralc6sxw67`) | **active through 2026-06-16** (checked at tip) |
| Old Collector (`addr1wyr4927...`) | dead after 2026-05-28 15:15 UTC |

Reference-script `block_time` on Koios: old CDP spend ref created 2024-10-01; all v3 refs created 2026-05-27 22:1x UTC.
The `reference_script.hash` returned by Koios for each v3 ref matches the v3 config `validatorHashes` exactly.

---

## 2. The diff: VX (current `.env.mainnet`) → V3

### 2a. Spend/validator script hashes — ALL CHANGED ❌

| Validator | VX (env) | V3 |
|---|---|---|
| CDP spend | `0805d8541db33f4841585fed4c3a7e87e2ff7018243038f06ceb660c` | `ff0b10bff20e4b68b491492e5ba6c8048a704763b0a45ce2995da0be` |
| CDP Creator | `0910f79461a71f74782dcd450f22b0c2cac31ab036b66c9219355d99` | `95c1458dad9dfddfab22d163fccbb9a8de483ace1e1c990b1f5e3eff` |
| Collector | `0752abd65a0c983bfb1c9c3880cc632c099ba3adb2fe307afb4bbd9c` | `68f23cd9befc2a74a00620b49d66a569cd130b290cd703e5c11891fd` |
| Stability Pool | `88e0299018563dd10c4860d9f34eda56fdb77f302da0e3980620535c` | `1c53ed6f616687b340ac83072ec65a9787583c01d6bae0314e1d61d0` |
| Staking | `a23793f529179e09cefb3c37fc6ae081e0e99e99be5cdb55a00941a5` | `112996eb011f20eacc28ded86cde177a53ed22264e0131199c983a7e` |

**V3 enterprise (`addr1w…`) script addresses (bech32, mainnet, header `0x71`):**
- CDP spend: `addr1w8lsky9l7g8yk695j9yjukaxeqzg5uz8vwc2gh8zn9w6p0sy523pp`
  ⚠️ actual CDP UTxOs use the `addr1z…` form (script payment + **user** stake key), not the enterprise form — verify which form tx3 needs for the spend party.
- CDP Creator: `addr1wx2uz3vd4kwlmhatytgk8lxthx5dujp6ec0pexgtra0ralc6sxw67`
- Collector: `addr1w950y0xehm7z5a9qqcstf8tx545u6yct9yxdwql9cyvfrlg4c5nk6`
- Stability Pool: `addr1wyw98mt0v9ng0v6q4jpswtkxt2tcwkpuq8tt4cp3fcwkr5qpqnput`
- Staking: `addr1wygjn9htqy0jp6kv9r0dsmx7zaa98mfzye8qzvgenjvr5ls2dl3wc`

(All addresses derived with the bech32 script in §6; the script self-checks by reproducing the
old CDP-spend address `addr1wyyqtkz5rken7jzptp076np606r79lmsrqjrqw8sdn4kvrqewrkdg` from `.env.mainnet`.)

### 2b. Reference script UTxOs

**Validator refs — NEW (all created 2026-05-27, index 0):**

| `.env.mainnet` var | V3 tx hash (index 0) |
|---|---|
| `CDP_SPEND_REF` | `f1275d84896bff1e557b7dc7d9cd4030fcd5539ae5ec99c3dd97d818c75c950a` |
| `CDP_CREATOR_REF` | `528a870ae53cd77dde8ace624ffd25610057e059466ded1d5d525d60141bf667` |
| `COLLECTOR_REF` | `4dfcc06496c2aee494909d7508a959d29465061db4cefb26ce331c971a9f606d` |
| `STABILITY_POOL_REF` | `8b76f66191bc65f0a7e4106b85a95b5524329a00049932f75faebdce4651d249` |
| `STAKING_REF` | `03a6b59ef3ca62527282aaf3461f374009392eb8a79aad635196077c17e74b28` |

**Mint-policy refs — mostly UNCHANGED (policies didn't change, see 2c):**

| `.env.mainnet` var | VX (env) | V3 |
|---|---|---|
| `CDP_NFT_MINT_REF` | `c0a4c2ad…fb7c21f8#0` | `c0a4c2ad340da8686c723a21b0a029aefee650fcaf5ef964742f499efb7c21f8#0` — **SAME** |
| `STAKING_POSITION_MINT_REF` | `71dc6b81…1450b4e7#0` | `71dc6b81e8832192bb28ecbc6a4f71b6e0dc0407c708f169020804371450b4e7#0` — **SAME** |
| `IASSET_MINT_REF` | `99329591…09a34107#0` | `iAssetTokenRef = 0c55ef7cdbc088057a76e1084aa9cc8dbd6350a720deceee490ce4cc8fc96a98#0` — **NEW UTxO, same policy** (verify it carries `f66d78b4…`; old UTxO likely still valid) |

Other V3 validator refs (not currently in our `.env`, may be needed for new flows — Pyth/gov/treasury/redeem):
`cdpRedeemValidatorRef 026712fd…`, `iassetValidatorRef cb580ecb…`, `interestCollectionValidatorRef 78ed00ff…`,
`robValidatorRef 61c3d17a…`, `executeValidatorRef 41ff5fad…`, `treasuryValidatorRef 197fce64…`,
`governanceValidatorRef c2275e7c…`, `versionRegistryValidatorRef 40f186ab…`,
`pollManagerValidatorRef 57f2c221…`, `pollShardValidatorRef 7fed3419…`, `stableswapValidatorRef 5a3581c7…`.
Auth-token refs: `cdpAuthTokenRef c0a4c2ad…`, `iAssetAuthTokenRef 0c55ef7c…`,
`collateralAssetTokenRef 63200f84ff3a35590a78b5c261a70b608d6a26cf9f36b99cdaf75be3543bc318#0`,
`stakingTokenRef 71dc6b81…`.

### 2c. Minting policies / token identities — UNCHANGED ✅

These are identical in VX and V3 (asset identity preserved — iUSD/iBTC/iETH/iSOL keep their policy IDs):

| Token | Policy ID | Name (hex) |
|---|---|---|
| iAsset mint (`IASSET_POLICY_ID` / cdpAssetSymbol) | `f66d78b4a3cb3d37afa0ec36461e51ecbde00f26c8f0a68f94b69880` | per-asset (iUSD `69555344`, …) |
| CDP NFT (`CDP_NFT_POLICY_ID` / cdpAuthToken) | `708f5e6d597fc038d09a738d7be32edd6ea779d6feb32a53668d9050` | `CDP` = `434450` |
| iAsset config NFT (iAssetAuth) | `97da12de04a6b527cc3b3469c5e5485cf258dfd1021f12e728f2e714` | `IASSET` = `494153534554` |
| INDY | `533bb94a8850ee3ccbe483106489399112b74c905342cb1792a797a0` | `INDY` = `494e4459` |
| Staking position token | `fd0d72fafee1d230a74c31ac503a192abd5b71888ae3f94128c1e634` | `STAKING_POSITION` |
| Staking Manager NFT | `24b458412c2a7f9acb9c53c7ec4325b36806912ed56d2f91bfcf4d26` | `STAKING_MANAGER_NFT` |
| Stability Pool NFT | `3f28fb7d6c40468262dffb1c3adb568b342499826b664d940085d022` | `STABILITY_POOL` |

⚠️ `CDP_CREATOR_POLICY_ID` (`735b3714…` in our env) was NOT confirmed in the V3 config dump — re-verify whether the CDP creator auth-token policy changed.

### 2d. NEW in V3 (not present in VX / our env) — likely behavior changes

1. **Pyth oracles.** V3 has a `pythConfig` block; price comes from **Pyth Network** feeds, replacing the
   old custom oracle-NFT system (`eedb4a24…:iUSD_INTEREST`, etc., documented in `investigation/indigo-research.md`).
   - Pyth State policy: `c935c937d0deda8975142c7b77aeef8f8cd48791e89a8ca7a0edc154` (token "Pyth State").
   - Feeds (priceFeedId): iUSD=inverse(16), iBTC=1÷16, iETH=2÷16, iSOL=6÷16, iJPY=340⁻¹÷16, iEUR=327÷16,
     plus NIGHT- and USDCx-denominated variants (see 2d.3).
   - **Impact:** `create_cdp` / `adjust_cdp_*` reference inputs change — they now read Pyth price UTxOs.
2. **New iAssets:** iJPY, iEUR added (on top of iUSD/iBTC/iETH/iSOL).
3. **Multi-collateral.** V3 introduces `collateralAssetAuthToken` (`b7d412f51a05bacbb34693bcac61efa935a8717d4b07801a115936ff`,
   name `COLLATERAL_ASSET`) and Pyth feeds denominated in **NIGHT** and **USDCx** — i.e. collateral is no
   longer ADA-only. The CDP datum/redeemer likely carries a collateral-asset field now.
4. New subsystems with refs/hashes: governance, treasury, pollManager/pollShard, execute, ROB,
   interestCollection, stableswap, versionRegistry, version-record/upgrade tokens.

---

## 3. Risks — things that probably changed *structurally* (not just hashes)

Re-pointing the env is necessary but **not sufficient**. Re-verify these against fresh V3 on-chain txs
before trusting any tx:

1. **Staking datums.** April research noted the "VX staking upgrade pending"; all staking UTxOs then used
   empty `Constr(0,[])`. The staking hash changed in V3, so the upgrade is likely **now active** →
   `StakingPosition { owner, lockedAmount, snapshot }` instead of empty. This would break
   `create_staking` / `adjust_staking` / `unstake` datum construction.
2. **CDP datum/redeemer.** Multi-collateral (2d.3) likely added a collateral-asset field to the CDP datum
   and/or `CreateCDP` / `AdjustCDP` redeemers. Re-derive from a real V3 `create_cdp` tx.
3. **Oracle reference inputs.** Pyth (2d.1) changes which reference inputs `create_cdp`/`adjust` consume,
   and possibly the interest-accumulator computation (see `indigo-research.md` §Interest Accumulator).
4. **Stability Pool.** SP hash changed; re-verify the request/process `AccountAction` structure still holds.
5. **CDP_CREATOR auth-token policy** — confirm whether it changed (see 2c ⚠️).

---

## 4. Step-by-step plan

1. **Pull the full raw V3 config** and save it for reference:
   `curl -s https://config.indigoprotocol.io/mainnet/mainnet-system-params-v3.json -o investigation/v3-system-params.json`
   (Keep it in the repo so the mapping is auditable.)
2. **Map V3 config → `.env.mainnet` vars.** Update every value in §2 (validator hashes/addresses,
   validator refs; keep the unchanged mint policies and the unchanged mint refs). Decide the CDP-spend party
   form (`addr1w` vs `addr1z`).
3. **Re-derive addresses** with the bech32 script in §6; confirm the self-check still reproduces the old
   CDP-spend address before trusting the new ones.
4. **Verify on-chain** (Koios, §6): every new ref UTxO is unspent and its `reference_script.hash` matches
   the V3 `validatorHashes`; the V3 CDP_CREATOR is the actively-used one.
5. **Fetch 2–3 recent V3 txs** for each user-facing flow (create_cdp, adjust_cdp mint/burn, close_cdp,
   create/adjust/close SP account, create/adjust/unstake staking) and re-derive datum + redeemer CBOR.
   Pay special attention to staking datums, CDP multi-collateral fields, and Pyth reference inputs (§3).
6. **Update `main.tx3`** for any datum/redeemer/reference-input changes. Then update `trix.toml` mainnet
   profile if needed.
7. **`trix check` → `trix build`** → copy `.tx3/tii/main.tii` to the server's `protocols/indigo.tii` → restart.
8. **Update `invoke-args/*.json`** and re-run the comparison (`invoke-args/comparison-report.md`) against V3
   reference txs; aim for the same ~100% structural match we had on VX.
9. **Update docs/memory** when done (memory: `indigo-v3-migration.md`).

> Consider running this through the `analyze-protocol` skill — it's the same flow that produced the VX
> implementation, now re-targeted at V3.

---

## 5. Full V3 reference data (validatorHashes + policy IDs)

**validatorHashes:** cdp `ff0b10bf…2995da0be` · cdpCreator `95c1458d…1f5e3eff` · collector `68f23cd9…c11891fd` ·
stabilityPool `1c53ed6f…4e1d61d0` · staking `112996eb…9c983a7e` · gov `adc336ca…1dc1a64b` ·
treasury `46ffe1d1…39622abd` · execute `0e764163…ca1974ce` · interestCollection `2cac220a…517ffc04` ·
pollManager `2c64c712…b6c13c68` · pollShard `814dc78c…b01c5a03` · rob `d5c13605…b3bc857c` ·
iasset `a9c613a0…1a920028` · stableswap `245425bb…aa2bc5a3` · versionRegistry `ea84d625…850ae744`.

**Policy IDs:** (unchanged ones in §2c) plus Pyth State `c935c937…a0edc154` · collateralAsset `b7d412f5…115936ff` ·
gov NFT `2fccae8b…4eaf8cce` · upgrade `ca72f111…0299c2a8` · versionRecord `d626ddf3…c18c7c2e` ·
pollManager `f9b162ea…4c22dd4f` · spEpoch `98ebc5df…16214720` · spAccount `443c51db…96a50813` ·
interestAdmin `47fa499d…74622d36` · daoIdentity `07d3770b…0552558d`.

**startTime:** slot `77419576`, block header `2906cb8553ad8d58befa954ba077709ef3ef9dcb59d2ab95b8e6bd25dd36e3a5`.

---

## 6. Re-runnable verification commands

**Koios — is a ref UTxO live + what script does it carry (also gives block_time):**
```bash
curl -s -X POST https://api.koios.rest/api/v1/utxo_info \
  -H "Content-Type: application/json" \
  -d '{"_utxo_refs":["f1275d84896bff1e557b7dc7d9cd4030fcd5539ae5ec99c3dd97d818c75c950a#0"],"_extended":true}' \
  | python3 -c 'import sys,json;u=json.load(sys.stdin)[0];print("spent",u["is_spent"],"refscript",(u.get("reference_script") or {}).get("hash"))'
```

**Koios — recent activity after a block height (detect dead vs live address):**
```bash
TIP=$(curl -s https://api.koios.rest/api/v1/tip | python3 -c 'import sys,json;print(json.load(sys.stdin)[0]["block_no"])')
curl -s -X POST https://api.koios.rest/api/v1/address_txs -H "Content-Type: application/json" \
  -d "{\"_addresses\":[\"addr1wx2uz3vd4kwlmhatytgk8lxthx5dujp6ec0pexgtra0ralc6sxw67\"],\"_after_block_height\":$((TIP-700000))}" \
  | python3 -c 'import sys,json,datetime;d=json.load(sys.stdin);t=sorted(x["block_time"] for x in d);print(len(d),"txs, last",datetime.datetime.fromtimestamp(t[-1],datetime.UTC) if d else "none")'
```

**bech32 — script hash → mainnet enterprise (`addr1w…`) address (self-checking):**
```python
CHARSET="qpzry9x8gf2tvdw0s3jn54khce6mua7l"
def polymod(v):
    g=[0x3b6a57b2,0x26508e6d,0x1ea119fa,0x3d4233dd,0x2a1462b3];c=1
    for x in v:
        b=c>>25;c=((c&0x1ffffff)<<5)^x
        for i in range(5):c^=g[i] if((b>>i)&1) else 0
    return c
def hrp(h):return[ord(x)>>5 for x in h]+[0]+[ord(x)&31 for x in h]
def cb(bs):
    acc=bits=0;r=[]
    for b in bs:
        acc=(acc<<8)|b;bits+=8
        while bits>=5:bits-=5;r.append((acc>>bits)&31)
    if bits:r.append((acc<<(5-bits))&31)
    return r
def addr(hexhash):
    bs=[0x71]+[int(hexhash[i:i+2],16) for i in range(0,len(hexhash),2)]
    d=cb(bs);pm=polymod(hrp('addr')+d+[0]*6)^1
    return 'addr1'+''.join(CHARSET[x] for x in d+[(pm>>5*(5-i))&31 for i in range(6)])
assert addr('0805d8541db33f4841585fed4c3a7e87e2ff7018243038f06ceb660c')=='addr1wyyqtkz5rken7jzptp076np606r79lmsrqjrqw8sdn4kvrqewrkdg'
print(addr('ff0b10bff20e4b68b491492e5ba6c8048a704763b0a45ce2995da0be'))  # v3 CDP spend
```

---

## 7. Open questions

- Is the VX deployment fully retired, or can legacy positions still be closed on it? (We confirmed no *new*
  activity after 2026-05-28, but legacy close/migration paths may still exist.)
- Did `CDP_CREATOR_POLICY_ID` change in V3? (Not confirmed — see 2c.)
- Exact CDP datum shape under multi-collateral, and exact Pyth reference-input layout for create/adjust.
- Whether tx3's current limitations (`investigation/tx3-limitations-indigo.md`) still block the Pyth
  price/interest math the way they did the old oracle math.
