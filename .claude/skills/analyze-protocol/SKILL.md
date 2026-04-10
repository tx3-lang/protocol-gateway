---
name: analyze-protocol
description: Analyze a Cardano protocol on-chain and implement it as a tx3 protocol with invoke-args and comparison report
argument-hint: [protocol-name] [optional: contract-address-or-policy-id]
---

# Analyze Cardano Protocol and Implement in TX3

Analyze the on-chain protocol and implement it as a tx3 protocol.

**Arguments provided:** $ARGUMENTS

## How to Start

1. If the user provided a protocol name, check if `protocols_tx3/<name>/` already exists with a scaffold (main.tx3, trix.toml). If so, read the existing files to understand what's already been done.
2. If a contract address or policy ID was provided, use it as the starting point for on-chain research.
3. If no address was provided, ask the user for one, or look for clues in the existing scaffold (addresses in comments, .env files, investigacion/ folder).
4. Check if there's an `investigacion/` folder with prior research — build on it rather than starting from scratch.

## Phase 1: On-Chain Research

Use the Koios REST API (https://api.koios.rest/api/v1/) to investigate the protocol.

### 1.1 Identify Contract Addresses and Scripts

Starting from the provided address or policy ID:

```
POST https://api.koios.rest/api/v1/address_info
POST https://api.koios.rest/api/v1/address_assets
POST https://api.koios.rest/api/v1/address_utxos (with _extended: true for reference scripts)
```

Find:
- All script addresses involved (validators, minting policies)
- Reference script UTxOs and their script hashes
- NFTs or tokens that identify protocol state (e.g., auth tokens, settings NFTs)
- Protocol settings/config UTxOs (reference inputs)

### 1.2 Analyze Real Transactions

Find recent transactions for the protocol:

```
POST https://api.koios.rest/api/v1/address_txs
POST https://api.koios.rest/api/v1/asset_txs (for tokens)
POST https://api.koios.rest/api/v1/asset_history (for minting events)
```

For each transaction type found:

```
POST https://api.koios.rest/api/v1/tx_info
POST https://api.koios.rest/api/v1/tx_cbor
```

Extract and document:
- **Inputs**: addresses, UTxO refs, assets consumed
- **Outputs**: addresses, values, native assets, inline datums
- **Minting**: policies, token names, quantities, redeemers
- **Reference inputs**: what config/state UTxOs are read
- **Redeemers**: constructor index and field values
- **Collateral**: present or not (indicates script execution)
- **Validity interval**: bounds if any
- **Metadata**: label and content
- **Inline scripts vs reference scripts**: how validators are provided

### 1.3 Decode Datums

For each inline datum found, decode the CBOR and identify:
- Number of fields
- Field types (Int, Bytes, Constr, List, Map)
- Semantic meaning of each field (by cross-referencing with GitHub source if available)
- Any discrepancies between GitHub source and on-chain reality

Document findings in `protocols_tx3/$0/investigacion/`.

### 1.4 Classify Values

Separate protocol configuration into layers:

| Layer | Description | Where it goes in tx3 |
|-------|-------------|---------------------|
| **Constants** | Same across all instances (token names, fixed strings) | `env {}` block |
| **Instance-level** | Shared across all markets/pools in one deployment | `env {}` or parties in `.env.mainnet` |
| **Per-market/pool** | Changes for each market, pool, or user interaction | tx params (JSON args) |
| **Per-call** | User address, amounts, prices | tx params (JSON args) |

## Phase 2: TX3 Implementation

### 2.1 Project Setup

```bash
mkdir -p protocols_tx3/$0/{invoke-args,investigacion}
```

Create `trix.toml`:
```toml
[protocol]
name = "$0"
scope = ""
version = "0.0.0"
description = ""
main = "main.tx3"

[ledger]
family = "cardano"
```

### 2.2 Write main.tx3

Structure the file as:
1. **Parties** — one per distinct address role
2. **Env block** — constants and instance-level config only
3. **Types** — datums, redeemers, helper types (match on-chain field count exactly)
4. **Transactions** — one per user action, ordered by complexity

Known tx3 limitations to account for:
- **Enum params not supported**: split txs into variants with hardcoded enum (e.g., `_yes`/`_no`)
- **Trailing commas required** in struct/variant constructors
- **Empty braces `{}` required** for unit variants (e.g., `BuyPos {}`)
- **Name collisions**: tx param names must not match type field names (causes lowering panic)
- **Empty env values need quotes**: `VAR=""` not `VAR=`
- **Nested metadata maps fail**: use simple strings for metadata values
- **Bytes in enum datum field**: wraps as ByteString, not raw Plutus Data

See `protocols_tx3/bodega-market/investigacion/tx3-custom-types-limitation.md` for full details.

### 2.3 Create .env.mainnet Profile

Only include values that are truly constant or instance-level. Per-market/per-call values go as tx params.

Party naming convention: `PartyName` in tx3 → `PARTYNAME` in .env (uppercase, no underscores added).
Dynamic parties (provided per call) go in JSON args as lowercase: `"user": "addr1..."`.

### 2.4 Create invoke-args JSON Files

One JSON per transaction. Include:
- Dynamic party addresses (e.g., `"user": "addr1..."`)
- All tx params with real mainnet values from Phase 1

## Phase 3: Testing & Comparison

### 3.1 Compile and Test

```bash
cd protocols_tx3/$0
trix check                    # syntax validation
trix invoke --skip-submit --profile mainnet --args-json-path invoke-args/<tx>.json
```

### 3.2 Find Test Wallets

For each tx, find a mainnet wallet that currently holds the required assets:

```
POST https://api.koios.rest/api/v1/address_info
POST https://api.koios.rest/api/v1/address_assets
```

Requirements:
- Enough ADA for the tx value + fees
- Required native tokens (if any)
- For collateral: needs a pure-ADA UTxO (multi-asset UTxOs fail collateral resolution)

### 3.3 Generate CBOR and Compare

For each tx:
1. Run `trix invoke --skip-submit --profile mainnet --args-json-path <args>.json`
2. Fetch the real on-chain tx CBOR: `POST https://api.koios.rest/api/v1/tx_cbor`
3. Compare field by field:
   - Output addresses, values, and assets
   - Datum fields (decode both CBORs and compare)
   - Minting, reference inputs, redeemers
   - Note expected differences: input UTxOs, fees, validity slots, CBOR encoding style

Expected differences that are NOT errors:
- **Input UTxOs**: different wallet, different UTxOs
- **Fees**: placeholder vs real execution cost
- **Validity slots**: tip_slot()+N vs historical absolute slots
- **CBOR encoding**: definite-length (`89`, `82`) vs indefinite-length (`9f...ff`) — semantically identical
- **Witnesses**: `--skip-submit` produces unsigned txs
- **Metadata content**: simple string vs complex (if tx3 nested metadata limitation applies)

### 3.4 Fix Discrepancies

If comparison reveals mismatches:
- Wrong datum field count → update type definition
- Wrong field values → check param mapping
- Missing outputs → update tx body
- Wrong pos_type/enum → verify enum constructor ordering matches on-chain
- Share tokens present/absent → verify the actual on-chain pattern

### 3.5 Document Results

Create `invoke-args/comparison-report.md` with:
- Reference transactions table (real on-chain tx hashes)
- Generated transactions table (all txs with status)
- Test wallets used
- Structural comparison per tx type
- Datum field-by-field comparison
- Expected differences explanation
- Protocol configuration (instance-level + per-market examples)
- How to reproduce (commands for each tx)

## Phase 4: Finalize

1. Run `trix build` to compile the `.tii`
2. Copy `.tx3/.tii/main.tii` to `protocols/<protocol-name>.tii`
3. Update comparison report with final status
4. Update project memory if needed

## Reference: Koios API Patterns

```bash
# Address info (balance, UTxO count)
curl -X POST https://api.koios.rest/api/v1/address_info \
  -H "Content-Type: application/json" \
  -d '{"_addresses": ["addr1..."]}'

# Address assets (native tokens)
curl -X POST https://api.koios.rest/api/v1/address_assets \
  -H "Content-Type: application/json" \
  -d '{"_addresses": ["addr1..."]}'

# Address UTxOs (with reference scripts)
curl -X POST https://api.koios.rest/api/v1/address_utxos \
  -H "Content-Type: application/json" \
  -d '{"_addresses": ["addr1..."], "_extended": true}'

# Transaction info
curl -X POST https://api.koios.rest/api/v1/tx_info \
  -H "Content-Type: application/json" \
  -d '{"_tx_hashes": ["txhash..."]}'

# Transaction CBOR
curl -X POST https://api.koios.rest/api/v1/tx_cbor \
  -H "Content-Type: application/json" \
  -d '{"_tx_hashes": ["txhash..."]}'

# Asset transaction history
curl -X POST https://api.koios.rest/api/v1/asset_txs \
  -H "Content-Type: application/json" \
  -d '{"_asset_policy": "policy...", "_asset_name": "name..."}'

# Asset mint/burn history
curl -X POST https://api.koios.rest/api/v1/asset_history \
  -H "Content-Type: application/json" \
  -d '{"_asset_policy": "policy...", "_asset_name": "name..."}'

# Asset holders
curl -X POST https://api.koios.rest/api/v1/asset_addresses \
  -H "Content-Type: application/json" \
  -d '{"_asset_policy": "policy...", "_asset_name": "name..."}'

# Address transactions (recent activity)
curl -X POST https://api.koios.rest/api/v1/address_txs \
  -H "Content-Type: application/json" \
  -d '{"_addresses": ["addr1..."], "_after_block_height": 13000000}'
```
