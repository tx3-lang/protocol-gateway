# B2 — Successful API Calls

This document demonstrates successful interactions with each of the selected protocols. Every transaction was built by the JSON-RPC API deployed at **`https://rpc.tx3.land/`** (which loads the `.tii` files compiled from the `.tx3` sources in this repository), signed with the user's wallet, and submitted to **mainnet**.

Each section contains:
- The HTTP `curl` against the public API (which builds and returns the tx CBOR).
- The local equivalent using `trix invoke` (same `.tx3`, same arguments).
- The JSON arguments that were sent.
- The transaction hash and a link to the public explorer (cexplorer.io).

> **Wallet used to sign and submit every tx:**
> `addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy`

> **How it works:** the `.tx3` sources live in `protocols_tx3/<protocol>/main.tx3`, are compiled with `trix build` to `.tii`, and the server (see `README.md`) loads them dynamically as JSON-RPC methods under `POST /{protocol}`. Protocol-specific values (script refs, contract addresses, etc.) come from the `mainnet` profile defined in each `trix.toml` and are baked into the `.tii`, so callers only need to pass the dynamic arguments.

> **List of protocols available on the API:**
> ```bash
> curl https://rpc.tx3.land/
> # {"protocols":["asteria-dev","partner-chain-protocol","strike-staking","bodega_market","fluid-aquarium","ticketing-2026","vyfi","indigo"]}
> ```

> ⚠️ **Reproducibility notice:** the JSON arguments captured below are the exact ones used to build each original transaction. **They are not meant to be copy-pasted and re-executed** — several of them contain UTxOs or values derived from the on-chain state at that specific moment, and replaying them as-is will fail:
>
> - **Indigo `create_staking`:** `manager_utxo` was already consumed by the documented tx. The current Manager UTxO must be fetched (script credential `3bd5f8ba...`) and `old_total_stake` and `current_snapshot_ada` recomputed from its datum.
> - **Bodega `buy_position_yes`:** `unit_price` and `total_lovelace` reflect the AMM price at the time of the swap. The price moves with every buy/sell on the market, so they must be re-read from the prediction UTxO datum before submitting.
> - **Strike `stake`:** `staked_at` is the millisecond timestamp used to build the tx; it goes into the datum and is validated against the tx validity range. It must be regenerated with the current timestamp on each invocation.
> - **Fluid `withdraw_tank`:** `tank_utxo` points to the output created in step 4 — already consumed by step 5. Any new withdrawal needs the UTxO of a live tank.
> - **Fluid `create_babel_tank`:** mostly static (depends only on the wallet and its ADA balance), should be re-runnable as long as the caller has ≥5 ADA available.
>
> The purpose of this document is to **demonstrate the successful calls** (with on-chain hashes as proof), not to serve as an interactive sandbox. To reproduce a tx from scratch, see `protocols_tx3/local_run/COMMANDS.md`, which contains the Koios snippets that fetch the variable values from the current on-chain state.

---

## 1. Strike Staking — `stake`

Stakes 5 STRIKE in the active Strike contract (`addr1z9yh4zcqs4gh78y...`), creating a new position with the user's wallet as owner.

**API call (`POST https://rpc.tx3.land/strike-staking`):**
```bash
curl -X POST https://rpc.tx3.land/strike-staking \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "stake",
    "params": {
      "staker": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "owner_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
      "amount": 5000000,
      "staked_at": 1776373888814
    }
  }'
```

**Local equivalent with `trix invoke`:**
```bash
cd protocols_tx3/strike-staking
trix invoke --profile mainnet --args-json-path ../local_run/02-strike-stake.json
```

**Arguments (`02-strike-stake.json`):**
```json
{
    "staker": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "owner_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
    "amount": 5000000,
    "staked_at": 1776373888814
}
```

**On-chain result:**
- Tx hash: `756abe04dc23ba9175719908c7d7ecc21316691f7b508e6bee1885810031c535`
- Explorer: https://cexplorer.io/tx/756abe04dc23ba9175719908c7d7ecc21316691f7b508e6bee1885810031c535

---

## 2. Indigo — `create_staking`

Creates an INDY staking position in the Indigo protocol, consuming the current Manager UTxO and Collector UTxO and producing new outputs with the updated snapshot/total_stake.

**API call (`POST https://rpc.tx3.land/indigo`):**
```bash
curl -X POST https://rpc.tx3.land/indigo \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "create_staking",
    "params": {
      "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "cdpuseraddr": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "owner_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
      "indy_amount": 5000000,
      "manager_utxo": "b63c74c2a339fc26113f6499f8c48e67a1d364096aabf82384220018152814c6#0",
      "old_total_stake": 12940012495622,
      "current_snapshot_ada": 1422553719171
    }
  }'
```

**Local equivalent with `trix invoke`:**
```bash
cd protocols_tx3/indigo
trix invoke --profile mainnet --args-json-path ../local_run/04-indigo-create-staking.json
```

**Arguments (`04-indigo-create-staking.json`):**
```json
{
    "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "cdpuseraddr": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "owner_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
    "indy_amount": 5000000,
    "manager_utxo": "b63c74c2a339fc26113f6499f8c48e67a1d364096aabf82384220018152814c6#0",
    "old_total_stake": 12940012495622,
    "current_snapshot_ada": 1422553719171
}
```

**On-chain result:**
- Tx hash: `c54778b4fcb6741eed0d96763328673b4fa9947e0c6e5f29a735197fa94c7279`
- Explorer: https://cexplorer.io/tx/c54778b4fcb6741eed0d96763328673b4fa9947e0c6e5f29a735197fa94c7279

---

## 3. Bodega Market — `buy_position_yes`

Buys 10 shares of the YES position on the **CC01_ADA_REACHES_060** market of the Bodega prediction market, paying `unit_price` per share plus fees (admin + batcher + envelope).

**API call (`POST https://rpc.tx3.land/bodega_market`):**
```bash
curl -X POST https://rpc.tx3.land/bodega_market \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "buy_position_yes",
    "params": {
      "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "positionscript": "addr1w9jw5wpd06f5v53sltrvxpkymraugehamf86r5z3vyl9jygxlhyt4",
      "user_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
      "user_stake_key": "7beef42339b93ab270f214070955a730cc269f1a435858b9915a157b",
      "project_info_ref": "fc914f41696c345b1a782e53ef6117c90aee1d7561d4442574a1380d40df71c3#0",
      "buy_amount": 10,
      "batcher_fee_amount": 700000,
      "admin_fee_percent": 200,
      "unit_price": 536556,
      "total_lovelace": 8265560
    }
  }'
```

> Note: on the API the protocol is exposed as `bodega_market` (underscore); in the repo the directory is `bodega-market` (hyphen).

**Local equivalent with `trix invoke`:**
```bash
cd protocols_tx3/bodega-market
trix invoke --profile mainnet --args-json-path ../local_run/05-bodega-buy-position.json
```

**Arguments (`05-bodega-buy-position.json`):**
```json
{
    "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "positionscript": "addr1w9jw5wpd06f5v53sltrvxpkymraugehamf86r5z3vyl9jygxlhyt4",
    "user_pkh": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
    "user_stake_key": "7beef42339b93ab270f214070955a730cc269f1a435858b9915a157b",
    "project_info_ref": "fc914f41696c345b1a782e53ef6117c90aee1d7561d4442574a1380d40df71c3#0",
    "buy_amount": 10,
    "batcher_fee_amount": 700000,
    "admin_fee_percent": 200,
    "unit_price": 536556,
    "total_lovelace": 8265560
}
```

**On-chain result:**
- Tx hash: `77bee9ce5f044fa34d314f041377ee2c192c99aea288a03f4e170aa6cd0c33ed`
- Explorer: https://cexplorer.io/tx/77bee9ce5f044fa34d314f041377ee2c192c99aea288a03f4e170aa6cd0c33ed

---

## 4. Fluid Aquarium — `create_babel_tank`

Creates a babel tank in Fluid Aquarium with 5 ADA of initial liquidity, registering the user as owner.

**API call (`POST https://rpc.tx3.land/fluid-aquarium`):**
```bash
curl -X POST https://rpc.tx3.land/fluid-aquarium \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "create_babel_tank",
    "params": {
      "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "tankuseraddr": "addr1z8uhynz89x08gh95758e6dkthtwdlplqzk5an8vj0hzwsenmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4asv50fln",
      "tank_ada": 5000000,
      "owner_payment_hash": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
      "owner_stake_hash": "7beef42339b93ab270f214070955a730cc269f1a435858b9915a157b",
      "empty_bytes": ""
    }
  }'
```

**Local equivalent with `trix invoke`:**
```bash
cd protocols_tx3/fluid-aquarium
trix invoke --profile mainnet --args-json-path ../local_run/06-fluid-create-babel-tank.json
```

**Arguments (`06-fluid-create-babel-tank.json`):**
```json
{
    "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "tankuseraddr": "addr1z8uhynz89x08gh95758e6dkthtwdlplqzk5an8vj0hzwsenmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4asv50fln",
    "tank_ada": 5000000,
    "owner_payment_hash": "27a1233dbb7fba96a53d82e9373d6a187f504fc34d210302b9def086",
    "owner_stake_hash": "7beef42339b93ab270f214070955a730cc269f1a435858b9915a157b",
    "empty_bytes": ""
}
```

**On-chain result:**
- Tx hash: `00b510615cc8f85d5184b5f79294a72938b80d08aa388a61638070a0fd36e076`
- Explorer: https://cexplorer.io/tx/00b510615cc8f85d5184b5f79294a72938b80d08aa388a61638070a0fd36e076

---

## 5. Fluid Aquarium — `withdraw_tank` (recovery)

Closes the babel tank created in the previous step and returns the 5 ADA to the owner. Demonstrates the full create → withdraw cycle against the contract.

**API call (`POST https://rpc.tx3.land/fluid-aquarium`):**
```bash
curl -X POST https://rpc.tx3.land/fluid-aquarium \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "withdraw_tank",
    "params": {
      "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
      "tankuseraddr": "addr1z8uhynz89x08gh95758e6dkthtwdlplqzk5an8vj0hzwsenmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4asv50fln",
      "tank_utxo": "00b510615cc8f85d5184b5f79294a72938b80d08aa388a61638070a0fd36e076#0"
    }
  }'
```

**Local equivalent with `trix invoke`:**
```bash
cd protocols_tx3/fluid-aquarium
trix invoke --profile mainnet --args-json-path ../local_run/recovery-fluid-withdraw-tank.json
```

**Arguments (`recovery-fluid-withdraw-tank.json`):**
```json
{
    "user": "addr1qyn6zgeahdlm49498kpwjdeadgv875z0cdxjzqczh800ppnmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4askhygsy",
    "tankuseraddr": "addr1z8uhynz89x08gh95758e6dkthtwdlplqzk5an8vj0hzwsenmam6zxwde82e8pus5quy4tfesesnf7xjrtpvtny26z4asv50fln",
    "tank_utxo": "00b510615cc8f85d5184b5f79294a72938b80d08aa388a61638070a0fd36e076#0"
}
```

> The `tank_utxo` referenced here is exactly the output created by the tx from step 4, which shows the full create → withdraw cycle.

**On-chain result:**
- Tx hash: `873919562b15cd70fe9ddc30b26e4154a078bb5c3e87cc5a47ad37d3f1af1bae`
- Explorer: https://cexplorer.io/tx/873919562b15cd70fe9ddc30b26e4154a078bb5c3e87cc5a47ad37d3f1af1bae

---

## Summary

| # | Endpoint | Method | Tx hash | Explorer |
|---|----------|--------|---------|----------|
| 1 | `https://rpc.tx3.land/strike-staking` | `stake` | `756abe04...0031c535` | [cexplorer](https://cexplorer.io/tx/756abe04dc23ba9175719908c7d7ecc21316691f7b508e6bee1885810031c535) |
| 2 | `https://rpc.tx3.land/indigo` | `create_staking` | `c54778b4...a94c7279` | [cexplorer](https://cexplorer.io/tx/c54778b4fcb6741eed0d96763328673b4fa9947e0c6e5f29a735197fa94c7279) |
| 3 | `https://rpc.tx3.land/bodega_market` | `buy_position_yes` | `77bee9ce...cd0c33ed` | [cexplorer](https://cexplorer.io/tx/77bee9ce5f044fa34d314f041377ee2c192c99aea288a03f4e170aa6cd0c33ed) |
| 4 | `https://rpc.tx3.land/fluid-aquarium` | `create_babel_tank` | `00b51061...fd36e076` | [cexplorer](https://cexplorer.io/tx/00b510615cc8f85d5184b5f79294a72938b80d08aa388a61638070a0fd36e076) |
| 5 | `https://rpc.tx3.land/fluid-aquarium` | `withdraw_tank` | `87391956...f1af1bae` | [cexplorer](https://cexplorer.io/tx/873919562b15cd70fe9ddc30b26e4154a078bb5c3e87cc5a47ad37d3f1af1bae) |

All five transactions were built by the JSON-RPC API at `https://rpc.tx3.land/` from the `.tx3` sources in this repository (compiled to `.tii`), signed with the user's wallet, and submitted to Cardano mainnet. The corresponding JSON arguments are version-controlled under `protocols_tx3/local_run/` for reproducibility. Each API response includes a `tx` field (CBOR hex ready to sign) and a `hash` field (the resulting tx id).
