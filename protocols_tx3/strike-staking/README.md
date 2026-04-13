# Strike Staking

tx3 protocol for [Strike Finance](https://github.com/strike-finance/staking-smart-contracts) staking on Cardano.

> **Status: WIP** — This tx3 off-chain implementation is under testing and may not cover all edge cases yet.

## Overview

Strike Finance allows users to lock STRIKE tokens in a staking contract and earn ADA rewards. On initial stake, a credential NFT pair (tracker token + owner identifier NFT) is minted. Both tokens are locked in the script UTxO — the owner NFT is **not** returned to the staker's wallet. Ownership is proven via the required signer matching the `owner_address_hash` in the datum.

## Transactions

| Transaction | Description |
|---|---|
| `stake` | Lock STRIKE tokens and mint credential NFTs |
| `add_stake` | Add more STRIKE to an existing stake position |
| `withdraw_stake` | Reclaim all STRIKE + accumulated ADA rewards, burning credential NFTs |

## Important Considerations

- **Network profile required:** The `env` block references on-chain addresses, policy IDs, and reference script UTxOs that must be configured per network (mainnet/testnet) in the trix profile.
- **Reference scripts:** Both spend and mint validators are consumed via reference scripts (`spend_script_ref`, `mint_script_ref`). These UTxOs must exist on-chain and be correctly referenced.
- **Owner NFT:** The owner NFT (minted with the staker's payment key hash as asset name) is locked in the script UTxO alongside the tracker token. It is **not** held in the staker's wallet. The `add_stake` transaction preserves it in the script output; `withdraw_stake` burns it.
- **Collateral:** All transactions require 5 ADA collateral from the staker.
- **Validity window:** The `stake` transaction uses a validity window of `tip_slot() + 200`.

## Caller Preparation

### `stake` / `add_stake`

- `staked_at_time: Int` — Current POSIX timestamp in **milliseconds**. tx3's `slot_to_time()` returns seconds, not milliseconds, so the caller must compute this value externally (e.g., current Unix time * 1000). This is needed because the datum spread workaround requires explicit field assignment.

### `add_stake`

- `staking_utxo: UtxoRef` — The existing staking position UTxO to add to. Must be queried on-chain.
- `additional_amount: Int` — Amount of STRIKE tokens to add.

### `withdraw_stake`

- `staking_utxo: UtxoRef` — The staking position UTxO to withdraw from. The owner NFT and tracker token are burned from the script input.

## tx3 Limitations

Minor tx3 bugs required workarounds (param/field name collisions, datum spread failures, `slot_to_time()` returning seconds instead of milliseconds). None are blocking — all 3 transactions are fully functional. For details, see [tx3-limitations-strike-staking.md](tx3-limitations-strike-staking.md).

## CBOR Verification

All transactions have been compared against real on-chain transactions from the active contract. See `invoke-args/comparison-report.md` for the full field-by-field comparison.

## Smart Contract

- PlutusV3
- Source: [strike-finance/staking-smart-contracts](https://github.com/strike-finance/staking-smart-contracts)
