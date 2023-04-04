# DAO Core Pallet

Create a DAO and run.

## Overview
This module contains functionality to create, manage and destroy a DAO alongside with token issuance.
It acts as a central actor of the node and provides configuration features and smart contract hook points
to fine-tune and customize a DAO with great freedom.

## Interface

### Dispatchable Functions
- `create_dao`: Create a DAO, initially the owner will be the creator. This can be released to a multisig account during setup.
- `destroy_dao`: Remove a DAO from the pallet, requires to destroy the asset first if a token has been issued.
- `issue_token`: Issue a token for the DAO.
- `set_metadata`: Configure a link to IPFS or a CDN alongside with a hash for a structured JSON file.
- `change_owner`: Transfer ownership of a DAO to a new owner.
