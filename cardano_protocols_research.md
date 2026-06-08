# Cardano Protocols Research for Transaction Builder

**Date:** March 2026  
**Objective:** Evaluate the availability of documentation, open source code, plutus/blueprint files, and integration examples with transaction builders (MeshJS, Lucid, or others) for the following protocols:

---

## Executive Summary

| Protocol | Open Source | Plutus/Blueprint | Dev Docs | SDK/Transaction Builder | Integration Difficulty |
|-----------|-------------|-------------------|----------|-------------------------|---------------------------|
| Fluid (Aquarium) | ✅ Partial | ⚠️ Not confirmed | ✅ Yes | ❌ Not official | High |
| Strike Finance | ✅ Yes (Aiken) | ✅ Yes (blueprint) | ✅ Yes | ⚠️ Proprietary SDK | Medium |
| Bodega Market | ✅ Yes | ⚠️ Possible | ✅ Yes | ❌ Not official | High |
| Indigo Protocol | ✅ Yes (BUSL-1.1) | ⚠️ Not published | ✅ Yes | ⚠️ Proprietary SDK | High |
| VyFi | ⚠️ Partial | ⚠️ Partial registry | ⚠️ Limited | ❌ Not official | Very High |
| Moneta (USDM) | ❌ No | ❌ No | ⚠️ Minimal | ❌ Not applicable | N/A (Native) |

---

## 1. Fluid Tokens — Aquarium Protocol

### What is it?
Aquarium is FluidTokens' protocol that allows paying transaction fees with native tokens instead of ADA. It is the first decentralized fee market on Cardano. It also enables **automatic transactions** (Scheduled Transactions) through a network of validators that execute transactions when predefined conditions are met.

### Open Source
- **Aquarium Node (Java):** [github.com/FluidTokens/ft-aquarium-node](https://github.com/FluidTokens/ft-aquarium-node)
  - Implemented in Java with Yaci Store
  - Indexes UTxOs of the Aquarium contracts
  - Processes Scheduled Transactions when conditions are met
- **Smart Contracts MVP:** [github.com/FluidTokens/ft-aquarium-automatic-sc-mvp](https://github.com/FluidTokens/ft-aquarium-automatic-sc-mvp)
  - MVP version of the Aquarium rules
- The FluidTokens organization has **16 repositories** on GitHub

### Documentation
- **Official docs:** [docs.fluidtokens.com/cardano/aquarium/](https://docs.fluidtokens.com/cardano/aquarium/)
- **Developer Portal:** [github.com/FluidTokens/developer-portal](https://github.com/FluidTokens/developer-portal)

### Plutus / Blueprint Files
- The smart contracts are listed as open source according to the official documentation
- **No published `plutus.json` or blueprint file was found directly** in the repos found
- The node contracts use Blockfrost to interact with the chain

### Transaction Builder Integration
- **No official integration with MeshJS or Lucid exists**
- To integrate, you must:
  1. Query the Aquarium endpoint to obtain the available Tanks
  2. Build a transaction that includes the necessary Tanks (the smart contract verifies that the required tokens are sent)
  3. Use MeshJS or Lucid to build said transaction

### Technical Notes
```
Main components:
- FeeTanks: UTxOs with ADA to sponsor fees
- Aquarium Lambdas: conditions under which ADA from the tanks can be spent
- Validators: network of operators with 30k FLDT staked
- Parameters: ADA/token ratio (static or dynamic via oracle)
```

### Resources
- Docs: https://docs.fluidtokens.com/protocols/aquarium/
- GitHub Org: https://github.com/fluidtokens

---

## 2. Strike Finance

### What is it?
Decentralized derivatives protocol on Cardano. It offers trading of **options, forwards, and perpetual contracts** (futures with no expiration date). Strike V1 launched on mainnet in May 2025 using the original GMX model, where liquidity providers act as counterparty to the traders.

### Open Source
Strike Finance is **fully open source**. Its GitHub repositories:
- **Perpetuals:** [github.com/strike-finance/perpetuals-smart-contracts](https://github.com/strike-finance/perpetuals-smart-contracts)
- **Forwards:** [github.com/strike-finance/forwards-smart-contracts](https://github.com/strike-finance/forwards-smart-contracts)
- **Options:** [github.com/strike-finance/options-smart-contracts](https://github.com/strike-finance/options-smart-contracts)
- **Staking:** [github.com/strike-finance/staking-smart-contracts](https://github.com/strike-finance/staking-smart-contracts)
- **SDK v1:** [github.com/strike-finance/strike-sdk-v1](https://github.com/strike-finance/strike-sdk-v1)

### Smart Contract Language
The contracts are written in **Aiken** (modern Cardano language that compiles to Plutus Core). Requires Aiken installed in the PATH:
```bash
# Add Aiken to the PATH in ~/.zshrc or ~/.bashrc
```

### Plutus / Blueprint Files
- By using **Aiken**, a `plutus.json` file (blueprint) is automatically generated upon compilation
- The blueprint contains the validator definitions, datum/redeemer types, and script hashes
- Aiken repos typically include `plutus.json` in the root or in the `build/` directory

### SDK / Transaction Builder
- **Strike SDK v1** available on GitHub
- Allows interacting with the contracts from JavaScript/TypeScript
- Has integration with **hummingbot** for trading bots

### Audit
- Audit of the perpetuals contracts performed and published in the repo

### Documentation
- **Official docs:** [docs.strikefinance.org](https://docs.strikefinance.org/)
- Has detailed documentation of the contracts

### Technical Notes
```
Perpetuals model (GMX-style):
- Traders open long/short positions with leverage
- Required collateral (can be the underlying asset or stablecoin)
- Liquidation when collateral falls to a % of its original value
- STRIKE as additional collateral (burned on liquidation)
- Automated stop loss / take profit
- Funding rate to keep the perpetual price aligned with spot
```

---

## 3. Bodega Market

### What is it?
A **decentralized prediction markets** platform on the Cardano blockchain. It allows users to create, trade, and resolve prediction markets on real-world events (sports, politics) and on-chain events (price of ADA, MIN, SNEK, etc.).

### Open Source
- **Smart Contracts V2:** [github.com/bodega-market/bodega-market-smart-contracts-v2](https://github.com/bodega-market/bodega-market-smart-contracts-v2)
- **Smart Contracts V1:** [github.com/bodega-market/bodega-market-smart-contracts](https://github.com/bodega-market/bodega-market-smart-contracts)
  - Note: V1 still in alpha, not recommended for production
- **Docs:** [github.com/bodega-market/bodega-market-docs](https://github.com/bodega-market/bodega-market-docs)

### Plutus / Blueprint Files
- The V2 contract is documented with deployment instructions
- **No explicit public blueprint file was found** in the searches
- V1 is in alpha and subject to breaking changes

### Protocol Flow (per V2)
```
1. Protocol setup:
   - Mint of settings tokens and manager authentication
   - Sending to corresponding script addresses
   - Mint of reference script tokens

2. Project setup:
   - Mint of authentication tokens
   - Sending to project info UTxO with pledge
   - Transfer of open fee to treasury

3. User participation:
   - User sends payment tokens to script address with datum
   - Batcher collects positions and applies them to the project
   - User receives proportional share tokens

4. Resolution:
   - Distribution of rewards between project creator and protocol
   - Ratio determined by share_ratio in the project info datum
```

### Documentation
- **Official docs:** [docs.bodegacardano.org](https://docs.bodegacardano.org)
- Has a protocol section with contracts, features, staking, etc.

### Transaction Builder Integration
- **No official integration with MeshJS or Lucid exists**
- Can be implemented using the V2 contracts as a reference

---

## 4. Indigo Protocol

### What is it?
An autonomous **synthetic assets** protocol on Cardano. It allows creating iAssets (synthetic assets) that replicate the price of real-world assets (iBTC, iETH, iUSD, etc.) using CDPs (Collateral Debt Positions) with ADA or stablecoins as collateral.

### Open Source
Indigo **open-sourced its code in April 2023**:
- **Smart Contracts V1:** [github.com/IndigoProtocol/indigo-smart-contracts](https://github.com/IndigoProtocol/indigo-smart-contracts)
  - **License: Business Source License 1.1 (BUSL-1.1)** — Not free for commercial use until it expires
- **SDK:** [github.com/IndigoProtocol/indigo-sdk](https://github.com/IndigoProtocol/indigo-sdk)

### Language Evolution
- **V1:** Written in **PlutusTx** with the Plutonomy optimizer
- **V2 (in development):** Migration to **Aiken** for better efficiency (lower CPU/memory usage)
  - Tests showed a 40-60% reduction in execution units

### Plutus / Blueprint Files
- The compiled V1 contracts are available in the repository
- The repo includes benchmarks in YAML with execution unit limits
- **For V2 (Aiken):** Will generate a blueprint automatically upon compilation

### SDK
- `indigo-sdk` available on GitHub
- Allows calculating INDY rewards and other interactions

### Documentation
- **Official docs:** [docs.indigoprotocol.io](https://docs.indigoprotocol.io) (inferred)
- Bug bounty program active since April 2023

### Technical Notes
```
Protocol components:
- CDPs: Minting of iAssets by collateralizing ADA (min 200%) or stablecoins (min 150%)
- Stability Pools: Liquidation of insolvent CDPs; stability providers receive collateral
- Governance (INDY): Voting on protocol parameters
- Liquid Staking: ADA in a CDP continues to generate staking rewards
- Oracles: iAsset prices fed by decentralized oracles
```

### BUSL Restriction
> ⚠️ **Important:** The BUSL-1.1 license prohibits commercial use of the V1 contracts until a conversion date. For integration into commercial products, verify the license terms or use V2 (Aiken) when available.

---

## 5. VyFi (VyFinance)

### What is it?
DeFi protocol on Cardano with multiple products: **DEX (AMM)**, BAR (redistributive mechanism), governance, lottery, and token/NFT Vaults. It also has an Auto-Harvester that manages yield farming using a Neural Net.

### Open Source
The VYFI organization on GitHub has repositories that are **limited in public scope**:
- **Cardano Contracts Registry:** [github.com/VYFI/cardano-contracts-registry](https://github.com/VYFI/cardano-contracts-registry)
  - Contains a registry of Cardano contracts
- **Metadata Registry Testnet:** [github.com/VYFI/metadata-registry-testnet](https://github.com/VYFI/metadata-registry-testnet)

### Plutus / Blueprint Files
- **No published plutus.json files or blueprints were found**
- The `cardano-contracts-registry` may contain script addresses but not the compiled source code
- The main DEX contracts **do not appear to be publicly open source**

### Documentation
- **Official docs:** [docs.vyfi.io](https://docs.vyfi.io)
- User documentation available but **without technical contract documentation for developers**

### Transaction Builder Integration
- **No official integration with MeshJS, Lucid, or others exists**
- Without access to the compiled contracts, integration would require:
  1. Reverse-engineering existing transactions
  2. Contacting the VyFi team directly
  3. Using their API (if one exists)

### Status
> ⚠️ **VyFi is the protocol with the least availability of developer resources among the 6 investigated.** No public contract code, blueprints, or SDK was found. Direct integration into a transaction builder would be significantly more complex than the other protocols.

---

## 6. Moneta (USDM)

### What is it?
**USDM** is Cardano's main fiat-backed stablecoin, issued by Moneta Digital LLC (a company registered as a Money Services Business with FinCEN in the U.S.). Each USDM is backed 1:1 by dollars in reserves (Fidelity + Western Asset Management). It also has a co-issuer in Europe: **NBX** (under MiCA regulation).

### Nature of the Token
USDM is a **Cardano Native Token** — **it is not a smart contract**. It is a native token created with a minting policy controlled by Moneta Digital. This is fundamentally different from the other protocols:

- It does not require smart contracts to transfer
- Transactions are simple native token transfers
- Only Moneta (and NBX in the EU) can mint/burn USDM

### Open Source
- **There is no open source code for the minting protocol**
- The mint/burn process is centralized and controlled by Moneta

### Plutus Files
- The minting policy may be a simple or multi-sig script
- **The minting policy script is not published**
- The token's Policy ID can be obtained by querying on-chain:
  - USDM Policy ID: identifiable in explorers such as cexplorer.io or pool.pm

### Transaction Builder Integration
As a native token, USDM can be used in normal Cardano transactions:
```typescript
// Example with MeshJS to send USDM
import { MeshTxBuilder } from "@meshsdk/core";

const tx = new MeshTxBuilder({ fetcher: provider });
await tx
  .txOut(recipientAddress, [
    { unit: "USDM_POLICY_ID" + "USDM", quantity: "1000000" } // 1 USDM (6 decimals)
  ])
  .complete();
```

### For minting/redemption
- Requires KYC and an account with Moneta (moneta.global) or NBX
- Minimum mint amount: $1,000 USD
- The process is off-chain (bank deposit → on-chain mint)

### USDM Policy ID
```
To obtain the current Policy ID of USDM, check:
- https://cardanoscan.io → search for "USDM"
- Or check directly in the Moneta documentation
```

---

## Recommendations for the Transaction Builder

### Protocols with the highest integration viability

#### 🟢 High Viability: Strike Finance
- Contracts in Aiken → blueprint generated automatically
- Proprietary SDK available
- Good contract documentation
- Fully open source

#### 🟡 Medium Viability: Bodega Market & Indigo Protocol
- Source code available but without SDKs for external builders
- Requires reading contracts and building manual integration
- Indigo has the BUSL-1.1 restriction

#### 🟡 Medium Viability: Fluid Tokens (Aquarium)
- The "fee sponsoring" logic is unique and valuable for UX
- There is a developer portal with API endpoints
- Integration requires querying the Aquarium API + building the tx

#### 🔴 Low Viability: VyFi
- No public contracts available
- No official SDKs
- Contact the team directly

#### ⚪ N/A: Moneta (USDM)
- It is a native token, integrates natively into any transaction builder
- Does not require smart contract logic to use USDM as a payment token

---

## Additional Transaction Builder Resources for Cardano

| Tool | Language | Link |
|------|----------|------|
| **MeshJS** | TypeScript | meshjs.dev |
| **Lucid** (deprecated → Lucid Evolution) | TypeScript | github.com/lucid-evolution |
| **PyCardano** | Python | pycardano.readthedocs.io |
| **cardano-serialization-lib** | Rust/WASM | github.com/Emurgo/cardano-serialization-lib |
| **Aiken** | Aiken/Plutus | aiken-lang.org |
| **Atlas** | Haskell | github.com/geniusyield/atlas |

---

## Viability Analysis with Tx3 (Reverse Engineering)

### What is Tx3?

**Tx3** is a DSL (Domain Specific Language) created by [TxPipe](https://txpipe.io/) to describe the interface of UTxO protocols on Cardano. It is the "OpenAPI" of the eUTxO world: it allows defining transaction templates as parameterized functions, and then generates code bindings in TypeScript, Rust, Go, or Python.

> Tx3 **does not replace the on-chain contracts** (those remain Aiken/PlutusTx). It only describes how to interact with them off-chain in a declarative way.

**Key language components:**
- `party` — transaction participants (wallet or script)
- `policy` — on-chain script (validator or minting policy)
- `record` — data structures for datums and redeemers
- `tx` — transaction template with inputs, outputs, and logic

**Tooling:**
- `trix` — CLI for init, build, test, and bindgen of Tx3 projects
- `tx3up` — ecosystem installer
- VSCode extension — syntax highlighting, diagrams, testing form
- Devnet — integrated local test network

### General Reverse Engineering Process for Tx3

For protocols without a public blueprint, the workflow would be:

1. **Identify script addresses** on-chain (from the protocol UI or documentation)
2. **Explore historical transactions** on CardanoScan or Cexplorer
3. **Decode CBOR datums** → infer types and fields
4. **Identify redeemers** → what actions each validator accepts
5. **Write the `record`s** in Tx3 with the inferred types
6. **Write the `tx` templates** replicating the observed UTxO patterns
7. **Reference the script** with `policy ScriptName = import(script.plutus)` or as a fixed address

### Viability by Protocol with Tx3

#### 🟢 Strike Finance — High viability

**Estimated effort: 1-2 days**

This is the ideal case for Tx3. The contracts are written in Aiken, which automatically generates a `plutus.json` with fully typed datum and redeemer types. With that blueprint, the `record`s and `tx`s in Tx3 can be mapped directly without real reverse engineering.

```
// Conceptual example of how the .tx3 would look
policy Perpetuals = import(build/perpetuals.plutus);

record OpenPositionDatum {
  trader: Bytes,
  collateral: Int,
  direction: Int,    // 0 = long, 1 = short
  leverage: Int,
}

tx openPosition(quantity: Int, leverage: Int) {
  input source {
    from: Trader,
    min_amount: quantity,
  }
  output position {
    to: Perpetuals,
    amount: Ada(quantity),
    datum: OpenPositionDatum { ... }
  }
}
```

**Advantages:**
- Blueprint with complete types available
- Proprietary SDK as a reference to validate the implementation
- Published audit makes it easier to understand the flows

---

#### 🟡 Indigo Protocol — Medium viability

**Estimated effort: 3-5 days**

The PlutusTx (Haskell) source code is published, which allows reading the datum and redeemer types directly. The architecture is complex (CDPs, Stability Pools, oracles, governance) but well documented in the repository. The biggest obstacle is the **BUSL-1.1 license** which restricts commercial use.

For V2 (migration to Aiken, in development), viability increases significantly since there will be an automatic blueprint.

**Blockers:**
- BUSL-1.1 license: verify terms before commercial integrations
- Multiple interrelated validators complicate the templates
- Decentralized oracles require additional referencing logic

---

#### 🟡 Bodega Market — Medium viability

**Estimated effort: 3-5 days**

The V2 contracts are on GitHub. However, the internal technical documentation (datum types, redeemer structure) is not detailed in the docs. It requires reading the source code to derive the structures. The **batcher** model (which aggregates user positions before applying them to the contract) introduces a more complex UTxO pattern to describe in Tx3.

**Blockers:**
- The batcher flow is not trivial to model in simple templates
- No generated blueprint; the types must be inferred from the code
- V1 in alpha not recommended; V2 is the version to use

---

#### 🟠 Fluid Tokens (Aquarium) — Medium-low viability

**Estimated effort: 5-8 days**

The FeeTanks logic has atypical flows: the contract verifies dynamic ADA/token ratios via oracles, and the "fee sponsoring" flow involves UTxOs from multiple coordinated parties. Without a public blueprint, real on-chain transactions would have to be analyzed to decode the datum structure. The developer portal with API endpoints helps to understand the available parameters, but it does not replace knowledge of the contract.

**Blockers:**
- No blueprint or contract code published explicitly
- Dynamic ratios via oracle require referencing external UTxOs
- The multiple-Tanks-per-transaction model is complex to parameterize in Tx3

---

#### 🔴 VyFi — Low viability

**Estimated effort: 2-3 weeks or more**

This is the case of "pure" reverse engineering. Without published contracts, blueprint, SDK, or technical documentation, the only option is to exhaustively analyze historical transactions in explorers, manually decode each CBOR datum, and infer the complete structure of the protocol. It is feasible in theory, but the time cost is disproportionate relative to the other protocols.

**Recommended alternative:** Contact the VyFi team directly to request technical documentation or access to the contracts before attempting reverse engineering.

---

### Tx3 Viability Summary Table

| Protocol | Tx3 Viability | Source of Types | Estimated Effort | Main Blocker |
|-----------|---------------|-----------------|-------------------|----------------------|
| Strike Finance | 🟢 High | Aiken blueprint (auto-generated) | 1-2 days | None relevant |
| Indigo Protocol | 🟡 Medium | PlutusTx code on GitHub | 3-5 days | BUSL-1.1 license |
| Bodega Market | 🟡 Medium | V2 source code on GitHub | 3-5 days | Batcher model |
| Fluid Tokens | 🟠 Medium-low | On-chain CBOR + API endpoints | 5-8 days | No public blueprint |
| VyFi | 🔴 Low | On-chain CBOR only | 2-3 weeks+ | No source code |
| Moneta (USDM) | ⚪ N/A | Native token (not applicable) | Hours | None |

### Recommended Implementation Order

1. **Strike Finance** — To learn the Tx3 flow with a clean, well-documented case
2. **Indigo Protocol** — If the BUSL-1.1 license is not a blocker; clear source code
3. **Bodega Market** — Code available; requires understanding the batcher model
4. **Fluid Tokens** — Useful for UX (fee sponsoring), but requires more on-chain analysis
5. **VyFi** — Only if the others are complete or if the team provides technical documentation

---

*Research conducted in March 2026. Cardano DeFi projects evolve rapidly; verify repositories directly for up-to-date information.*
