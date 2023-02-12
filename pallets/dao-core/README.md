# DAO Core Pallet

Create a DAO and run.


## Overview
This module contains functionality to create, manage and destroy a DAO alongside with token issuance.

It acts as a central actor of the node and will provide configuration features and smart contract hookpoints to fine tune
and customize a DAO with great freedom.

## Interface

### Dispatchable Functions
create_dao - create an DAO, initially the owner will be the creator. This can be released to a multisig account upon finishing the setup in the next milestone
destroy_dao - remove a DAO from the pallet, requires to destroy the asset first if a token has been issued
issue_token - issue a token for the DAO
set_metadata - configure a link to IPFS or a CDN alongside with a hash for a structured JSON file. We are creating a helper backend service to upload the data for you, or you can upload it by yourself.
