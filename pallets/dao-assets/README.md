# DAO Assets

A DAO share registry optimized for chain agnostic voting.

## Overview
This module contains functionality to manage assets issued for a DAO. The DAO Assets are oriented on the pallet-asset
but are enhancing functionality with checkpoint registry functions.

## Interface

### Dispatchable Functions
- `start_destroy`: Start the process of destroying a fungible asset class.
- `destroy_accounts`: Destroy all accounts associated with a given asset.
- `destroy_approvals`: Destroy all approvals associated with a given asset up to the max of the configured `RemoveItemsList`
- `finish_destroy`: Complete destroying asset and unreserve currency.
- `transfer`: Move some assets from the sender account to another.
- `transfer_keep_alive`: Move some assets from the sender account to another, keeping the sender account alive.
- `transfer_ownership`: Change the Owner of an asset.
- `set_team`: Change the Issuer and Admin of an asset.
- `set_metadata`: Set the metadata for an asset.
- `clear_metadata`: Clear the metadata for an asset.
- `approve_transfer`: Approve an amount of asset for transfer by a delegated third-party account.
- `cancel_approval`: Cancel all of some asset approved for delegated transfer by a third-party account.
- `transfer_approved`: Transfer some asset balance from a previously delegated account to some third-party account.
